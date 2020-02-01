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
use sulis_module::{area::tile::TerrainRules, area::MAX_AREA_SIZE, Module};

use crate::{AreaModel, EditorMode};

const NAME: &str = "terrain_picker";

pub struct TerrainPicker {
    brush_size_spinner: Rc<RefCell<Spinner>>,
    brush_size_widget: Rc<RefCell<Widget>>,
    cursor_sprite: Rc<Sprite>,

    cursor_pos: Option<Point>,
    brush_size: i32,
    grid_width: i32,
    grid_height: i32,

    terrain_rules: TerrainRules,
    cur_terrain: Option<usize>,
}

impl TerrainPicker {
    pub fn new() -> Rc<RefCell<TerrainPicker>> {
        let cursor_sprite = match ResourceSet::sprite(&Config::editor_config().cursor) {
            Err(_) => panic!(
                "Unable to find cursor sprite '{}'",
                Config::editor_config().cursor
            ),
            Ok(sprite) => sprite,
        };

        let terrain_rules = Module::terrain_rules();
        let brush_size = 4;
        let brush_size_spinner = Spinner::new(brush_size, 1, 20);
        let brush_size_widget = Widget::with_theme(brush_size_spinner.clone(), "brush_size");

        Rc::new(RefCell::new(TerrainPicker {
            brush_size_spinner,
            brush_size_widget,
            cursor_sprite,
            cursor_pos: None,
            brush_size,
            grid_width: terrain_rules.grid_width as i32,
            grid_height: terrain_rules.grid_height as i32,
            terrain_rules,
            cur_terrain: None,
        }))
    }
}

impl EditorMode for TerrainPicker {
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
        let (index, cur_terrain) = match self.cur_terrain {
            None => return,
            Some(i) => (i, model.tiles().terrain_kind(i).clone()),
        };

        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_tiles_within(
            &self.terrain_rules.border_layer,
            x_min - 1 * self.grid_width,
            y_min - 1 * self.grid_height,
            (self.brush_size + 2) * self.grid_width,
            (self.brush_size + 2) * self.grid_height,
        );
        model.remove_tiles_within(
            &self.terrain_rules.base_layer,
            x_min,
            y_min,
            self.brush_size * self.grid_width,
            self.brush_size * self.grid_height,
        );

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }

                model.set_terrain_index(x, y, Some(index));
                model.add_tile(&Some(model.tiles().gen_choice(&cur_terrain)), x, y);
            }
        }

        // add tiles in a larger radius than we removed -
        // we rely on the model to not add already existing tiles twice
        for yi in -2..self.brush_size + 2 {
            for xi in -2..self.brush_size + 2 {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }
                model.check_add_terrain_border(x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_all_tiles(
            x_min,
            y_min,
            self.brush_size * self.grid_width,
            self.brush_size * self.grid_height,
        );

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
                    continue;
                }

                model.set_terrain_index(x, y, None);
            }
        }
    }
}

impl WidgetKind for TerrainPicker {
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
        self.brush_size_widget
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (_, picker) = Widget::parent_mut::<TerrainPicker>(widget);

                let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                    None => panic!("Unable to downcast to spinner"),
                    Some(widget) => widget,
                };

                picker.brush_size = spinner.value();
            })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        for (i, terrain_kind) in Module::terrain_kinds().into_iter().enumerate() {
            let base_tile_id = format!(
                "{}{}{}",
                self.terrain_rules.prefix, terrain_kind.id, self.terrain_rules.base_postfix
            );

            let button = Widget::with_theme(Button::empty(), "terrain_button");
            button
                .borrow_mut()
                .state
                .add_text_arg("icon", &base_tile_id);

            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::direct_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let (_, picker) = Widget::parent_mut::<TerrainPicker>(&parent);
                    picker.cur_terrain = Some(i);
                }
            }));
            button.borrow_mut().state.add_callback(cb);

            scrollpane.borrow().add_to_content(button);
        }

        vec![
            self.brush_size_widget.clone(),
            brush_size_label,
            Widget::with_theme(scrollpane, "terrain"),
        ]
    }
}
