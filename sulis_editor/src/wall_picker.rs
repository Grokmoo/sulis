//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::config::Config;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_core::widgets::{Button, Label, ScrollDirection, ScrollPane, Spinner};
use sulis_module::{area::tile::WallRules, area::MAX_AREA_SIZE, Module};

use crate::{AreaModel, EditorMode};

const NAME: &str = "wall_picker";

pub struct WallPicker {
    level_widget: Rc<RefCell<Widget>>,

    brush_size_spinner: Rc<RefCell<Spinner>>,
    brush_size_widget: Rc<RefCell<Widget>>,
    cursor_sprite: Rc<Sprite>,

    cursor_pos: Option<Point>,
    level: i32,
    brush_size: i32,
    grid_width: i32,
    grid_height: i32,

    wall_rules: WallRules,
    cur_wall: Option<usize>,
}

impl WallPicker {
    pub fn new() -> Rc<RefCell<WallPicker>> {
        let cursor_sprite = match ResourceSet::sprite(&Config::editor_config().cursor) {
            Err(_) => panic!(
                "Unable to find cursor sprite '{}'",
                Config::editor_config().cursor
            ),
            Ok(sprite) => sprite,
        };

        let wall_rules = Module::wall_rules();
        let brush_size = 4;
        let brush_size_spinner = Spinner::new(brush_size, 1, 20);
        let brush_size_widget = Widget::with_theme(brush_size_spinner.clone(), "brush_size");

        let level = 1;
        let level_spinner = Spinner::new(level, 0, 5);
        let level_widget = Widget::with_theme(level_spinner, "level");

        Rc::new(RefCell::new(WallPicker {
            level,
            level_widget,
            brush_size_spinner,
            brush_size_widget,
            cursor_sprite,
            cursor_pos: None,
            brush_size,
            grid_width: wall_rules.grid_width as i32,
            grid_height: wall_rules.grid_height as i32,
            wall_rules,
            cur_wall: None,
        }))
    }
}

impl EditorMode for WallPicker {
    fn draw_mode(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        _model: &AreaModel,
        x_offset: f32,
        y_offset: f32,
        scale_x: f32,
        scale_y: f32,
        _millis: u32,
    ) {
        let gw = self.grid_width as f32;
        let gh = self.grid_height as f32;

        let mut draw_list = DrawList::empty_sprite();
        if let Some(pos) = self.cursor_pos {
            for y in 0..self.brush_size {
                for x in 0..self.brush_size {
                    let x = gw * x as f32 + pos.x as f32 + x_offset;
                    let y = gh * y as f32 + pos.y as f32 + y_offset;
                    draw_list.append(&mut DrawList::from_sprite_f32(
                        &self.cursor_sprite,
                        x,
                        y,
                        gw,
                        gh,
                    ));
                }
            }
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(Color::from_string("0F08"));
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (
            self.brush_size * self.grid_width,
            self.brush_size * self.grid_height,
        )
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        let x = if x % 2 == 0 { x } else { x + 1 };
        let y = if y % 2 == 0 { y } else { y + 1 };
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn mouse_scroll(&mut self, _model: &mut AreaModel, delta: i32) {
        let value = self.brush_size_spinner.borrow().value() - delta;
        self.brush_size_spinner.borrow_mut().set_value(value);
        self.brush_size = self.brush_size_spinner.borrow().value();
        self.brush_size_widget.borrow_mut().invalidate_layout();
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };

        for layer in self.wall_rules.down_layers.iter() {
            model.remove_tiles_within(
                layer,
                x_min - 3 * self.grid_width,
                y_min - 3 * self.grid_width,
                (self.brush_size + 6) * self.grid_width,
                (self.brush_size + 6) * self.grid_height,
            );
        }

        for layer in self.wall_rules.up_layers.iter() {
            model.remove_tiles_within(
                layer,
                x_min - 3 * self.grid_width,
                y_min - 3 * self.grid_height,
                (self.brush_size + 6) * self.grid_width,
                (self.brush_size + 6) * self.grid_height,
            );
        }

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }

                model.set_wall(x, y, self.level as u8, self.cur_wall);
                for ye in y..=y + self.grid_height {
                    for xe in x - 1..=x + self.grid_height {
                        model.set_elevation(2 * self.level as u8, xe, ye);
                    }
                }
            }
        }

        for yi in -7..self.brush_size + 5 {
            for xi in -5..self.brush_size + 5 {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }
                model.check_add_wall_border(x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };

        let iter = self
            .wall_rules
            .up_layers
            .iter()
            .chain(self.wall_rules.down_layers.iter());
        for layer in iter {
            model.remove_tiles_within(
                layer,
                x_min,
                y_min,
                self.brush_size * self.grid_width,
                self.brush_size * self.grid_height,
            );
        }

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }

                model.set_wall(x, y, 0, None);
                for ye in y..y + self.grid_height {
                    for xe in x..x + self.grid_height {
                        model.set_elevation(0, xe, ye);
                    }
                }
            }
        }
    }
}

impl WidgetKind for WallPicker {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.level_widget
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (_, picker) = Widget::parent_mut::<WallPicker>(widget);
                let spinner = Widget::downcast::<Spinner>(kind);
                picker.level = spinner.value();
            })));
        let level_label = Widget::with_theme(Label::empty(), "level_label");

        self.brush_size_widget
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (_, picker) = Widget::parent_mut::<WallPicker>(widget);
                let spinner = Widget::downcast::<Spinner>(kind);
                picker.brush_size = spinner.value();
            })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let no_wall_button = Widget::with_theme(Button::empty(), "no_wall_button");
        let mut all_buttons = Vec::new();
        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        for (i, wall_kind) in Module::wall_kinds().into_iter().enumerate() {
            let base_tile_id = format!(
                "{}{}{}",
                self.wall_rules.prefix, wall_kind.id, wall_kind.base_tile
            );

            let button = Widget::with_theme(Button::empty(), "wall_button");
            button
                .borrow_mut()
                .state
                .add_text_arg("icon", &base_tile_id);

            let no_wall_ref = Rc::clone(&no_wall_button);
            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::direct_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    no_wall_ref.borrow_mut().state.set_active(false);
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let (_, picker) = Widget::parent_mut::<WallPicker>(&parent);
                    picker.cur_wall = Some(i);
                }
            }));
            button.borrow_mut().state.add_callback(cb);
            all_buttons.push(Rc::clone(&button));
            scrollpane.borrow().add_to_content(button);
        }

        no_wall_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let (_, picker) = Widget::parent_mut::<WallPicker>(widget);
                picker.cur_wall = None;
                for button in all_buttons.iter() {
                    button.borrow_mut().state.set_active(false);
                }

                widget.borrow_mut().state.set_active(true);
            })));

        vec![
            self.level_widget.clone(),
            level_label,
            self.brush_size_widget.clone(),
            brush_size_label,
            no_wall_button,
            Widget::with_theme(scrollpane, "walls"),
        ]
    }
}
