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

use std::io::Error;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::config::CONFIG;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::{Point};
use sulis_module::{Module, area::MAX_AREA_SIZE, area::tile::{Tile, WallKind, WallRules}};
use sulis_widgets::{Button, Label, Spinner};

use {AreaModel, EditorMode};
use terrain_picker::EdgesList;

const NAME: &str = "wall_picker";

#[derive(Clone)]
pub struct WallTiles {
    pub id: String,
    fill_tile: Option<Rc<Tile>>,

    edges: EdgesList,
    extended: Vec<EdgesList>,
}

impl WallTiles {
    pub fn new(kind: WallKind, rules: &WallRules) -> Result<WallTiles, Error> {
        let fill_tile = match kind.fill_tile {
            None => None,
            Some(ref fill_tile) => {
                let fill_tile_id = format!("{}{}{}", &rules.prefix, &kind.id, fill_tile);
                match Module::tile(&fill_tile_id) {
                    None => {
                        warn!("No fill tile found for '{}'", kind.id);
                        None
                    }, Some(tile) => Some(tile),
                }
            }
        };

        let edges = EdgesList::new(&kind.id, &rules.prefix, &rules.edges)?;

        let mut extended = Vec::new();
        for prefix in kind.extended {
            let e = EdgesList::new(&prefix, &rules.prefix, &rules.edges)?;
            extended.push(e);
        }

        Ok(WallTiles {
            id: kind.id,
            edges,
            extended,
            fill_tile,
        })
    }
}

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
        let cursor_sprite = match ResourceSet::get_sprite(&CONFIG.editor.cursor) {
            Err(_) => panic!("Unable to find cursor sprite '{}'", CONFIG.editor.cursor),
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

    fn check_add_border(&self, model: &mut AreaModel, x: i32, y: i32) {
        let (self_elev, self_index) = match model.wall_at(x, y) {
            (elev, Some(index)) => (elev, index),
            (_, None) => return,
        };

        let tiles = model.wall_kind(self_index).clone();

        model.add_tile(&tiles.fill_tile, x, y);

        let gh = self.grid_height;
        let gw = self.grid_width;

        let n = self.is_border(model, self_elev, self_index, x, y, 0, -gh);
        let e = self.is_border(model, self_elev, self_index, x, y, gw, 0);
        let s = self.is_border(model, self_elev, self_index, x, y, 0, gh);
        let w = self.is_border(model, self_elev, self_index, x, y, -gw, 0);
        let nw = self.is_border(model, self_elev, self_index, x, y, -gw, -gh);
        let ne = self.is_border(model, self_elev, self_index, x, y, gw, -gh);
        let se = self.is_border(model, self_elev, self_index, x, y, gw, gh);
        let sw = self.is_border(model, self_elev, self_index, x, y, -gw, gh);

        if n && nw && w { model.add_tile(&tiles.edges.outer_nw, x - gw, y - gh); }

        if n && nw && ne { model.add_tile(&tiles.edges.outer_n, x, y - gh); }

        if n && ne && e { model.add_tile(&tiles.edges.outer_ne, x + gw, y - gh); }

        if e && ne && se { model.add_tile(&tiles.edges.outer_e, x + gw, y); }
        else if e && ne { model.add_tile(&tiles.edges.inner_ne, x + gw, y); }
        else if e && se {
            model.add_tile(&tiles.edges.inner_se, x + gw, y);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                model.add_tile(&ext.outer_s, x + gw, y + offset * gh);
            }
        }

        if w && nw && sw { model.add_tile(&tiles.edges.outer_w, x - gw, y); }
        else if w && nw { model.add_tile(&tiles.edges.inner_nw, x - gw, y); }
        else if w && sw {
            model.add_tile(&tiles.edges.inner_sw, x - gw, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                model.add_tile(&ext.outer_s, x - gw, y + offset * gh);
            }
        }

        if s && sw && se {
            model.add_tile(&tiles.edges.outer_s, x, y + gh);
            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                model.add_tile(&ext.outer_s, x, y + offset * gh);
            }
        }

        if s && se && e {
            model.add_tile(&tiles.edges.outer_se, x + gw, y + gh);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                model.add_tile(&ext.outer_se, x + gw, y + offset * gh);
            }
        }

        if s && sw && w {
            model.add_tile(&tiles.edges.outer_sw, x - gw, y + gh);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                model.add_tile(&ext.outer_sw, x - gw, y + offset * gh);
            }
        }
    }

    fn is_border(&self, model: &AreaModel, self_elev: i8, _self_index: usize, x: i32, y: i32,
                 delta_x: i32, delta_y: i32) -> bool {

        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { return false; }

        let (dest_elev, dest_index) = model.wall_at(x, y);

        match dest_index {
            None => true,
            Some(_) => dest_elev < self_elev,
        }
    }
}

impl EditorMode for WallPicker {
    fn draw(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x_offset: f32, y_offset: f32,
            scale_x: f32, scale_y: f32, _millis: u32) {
        let gw = self.grid_width as f32;
        let gh = self.grid_height as f32;

        let mut draw_list = DrawList::empty_sprite();
        if let Some(pos) = self.cursor_pos {
            for y in 0..self.brush_size {
                for x in 0..self.brush_size {
                    let x = gw * x as f32 + pos.x as f32 + x_offset;
                    let y = gh * y as f32 + pos.y as f32 + y_offset;
                    draw_list.append(&mut DrawList::from_sprite_f32(&self.cursor_sprite, x, y, gw, gh));
                }
            }
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(Color::from_string("0F08"));
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (self.brush_size * self.grid_width, self.brush_size * self.grid_height)
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

        for layer in self.wall_rules.layers.iter() {
            model.remove_tiles_within(layer,
                                      x_min - 3 * self.grid_width,
                                      y_min - 3 * self.grid_height,
                                      (self.brush_size + 6) * self.grid_width,
                                      (self.brush_size + 6) * self.grid_height);
        }

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

                model.set_wall(x, y, self.level as i8, self.cur_wall);
                for ye in y-1..y + self.grid_height + 1 {
                    for xe in x-1..x + self.grid_height + 1 {
                        model.set_elevation(2 * self.level as u8, xe, ye);
                    }
                }
            }
        }

        for yi in -4..self.brush_size+4 {
            for xi in -4..self.brush_size+4 {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }
                self.check_add_border(model, x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };

        for layer in self.wall_rules.layers.iter() {
            model.remove_tiles_within(layer, x_min, y_min,
                                      self.brush_size * self.grid_width,
                                      self.brush_size * self.grid_height);
        }

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

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
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.level_widget.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<WallPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast spinner"),
                Some(widget) => widget,
            };

            picker.level = spinner.value();
        })));
        let level_label = Widget::with_theme(Label::empty(), "level_label");

        self.brush_size_widget.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<WallPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.brush_size = spinner.value();
        })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let wall_content = Widget::empty("wall_content");
        for (i, wall_kind) in Module::wall_kinds().into_iter().enumerate() {
            let base_tile_id = format!("{}{}{}", self.wall_rules.prefix, wall_kind.id,
                                       wall_kind.base_tile);

            let button = Widget::with_theme(Button::empty(), "wall_button");
            button.borrow_mut().state.add_text_arg("icon", &base_tile_id);

            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let parent = Widget::get_parent(&parent);
                    let wall_picker = Widget::downcast_kind_mut::<WallPicker>(&parent);
                    wall_picker.cur_wall = Some(i);
                }
            }));
            button.borrow_mut().state.add_callback(cb);

            Widget::add_child_to(&wall_content, button);
        }

        vec![self.level_widget.clone(), level_label, self.brush_size_widget.clone(), brush_size_label,
          wall_content]
    }
}
