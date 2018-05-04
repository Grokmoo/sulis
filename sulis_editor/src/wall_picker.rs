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
use sulis_core::util::{unable_to_create_error, Point};
use sulis_module::{Module, area::tile::{Tile, WallKind, WallRules}};
use sulis_widgets::{Button, Label, Spinner};

use {area_model::MAX_AREA_SIZE, AreaModel, EditorMode};

const NAME: &str = "wall_picker";

#[derive(Clone)]
struct WallTiles {
    pub id: String,

    inner_nw: Rc<Tile>,
    inner_ne: Rc<Tile>,
    inner_sw: Rc<Tile>,
    inner_se: Rc<Tile>,

    outer_n: Rc<Tile>,
    outer_s: Rc<Tile>,
    outer_e: Rc<Tile>,
    outer_w: Rc<Tile>,
    outer_se: Rc<Tile>,
    outer_ne: Rc<Tile>,
    outer_sw: Rc<Tile>,
    outer_nw: Rc<Tile>,
}

impl WallTiles {
    pub fn new(kind: WallKind, rules: &WallRules) -> Result<WallTiles, Error> {
        let id = &kind.id.to_string();

        let inner_nw = WallTiles::get_edge(&rules, &id, &rules.inner_edge_postfix, &rules.nw_postfix)?;
        let inner_ne = WallTiles::get_edge(&rules, &id, &rules.inner_edge_postfix, &rules.ne_postfix)?;
        let inner_sw = WallTiles::get_edge(&rules, &id, &rules.inner_edge_postfix, &rules.sw_postfix)?;
        let inner_se = WallTiles::get_edge(&rules, &id, &rules.inner_edge_postfix, &rules.se_postfix)?;

        let outer_n = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.n_postfix)?;
        let outer_s = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.s_postfix)?;
        let outer_e = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.e_postfix)?;
        let outer_w = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.w_postfix)?;
        let outer_ne = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.ne_postfix)?;
        let outer_nw = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.nw_postfix)?;
        let outer_se = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.se_postfix)?;
        let outer_sw = WallTiles::get_edge(&rules, &id, &rules.outer_edge_postfix, &rules.sw_postfix)?;
        Ok(WallTiles {
            id: kind.id,
            inner_nw, inner_ne, inner_sw, inner_se, outer_n,
            outer_s, outer_e, outer_w, outer_se, outer_ne, outer_sw,
            outer_nw,
        })
    }

    fn get_edge(rules: &WallRules, id: &str, edge_postfix: &str, dir_postfix: &str) -> Result<Rc<Tile>, Error> {
        let tile_id = format!("{}{}{}{}", rules.prefix, id, edge_postfix, dir_postfix);

        match Module::tile(&tile_id) {
            None => {
                warn!("Wall edge tile with '{}', '{}' not found for '{}'", edge_postfix, dir_postfix, id);
                return unable_to_create_error("wall_tile", &id);
            }, Some(tile) => Ok(tile),
        }
    }
}

pub struct WallPicker {
    brush_size_spinner: Rc<RefCell<Spinner>>,
    brush_size_widget: Rc<RefCell<Widget>>,
    cursor_sprite: Rc<Sprite>,

    cursor_pos: Option<Point>,
    brush_size: i32,
    grid_width: i32,
    grid_height: i32,

    wall_rules: WallRules,
    cur_wall: Option<usize>,

    wall_kinds: Vec<WallTiles>,
    walls: Vec<Option<usize>>,
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

        let mut wall_kinds = Vec::new();
        for kind in Module::wall_kinds() {
            match WallTiles::new(kind, &wall_rules) {
                Err(_) => continue,
                Ok(wall) => wall_kinds.push(wall),
            }
        }

        Rc::new(RefCell::new(WallPicker {
            brush_size_spinner,
            brush_size_widget,
            cursor_sprite,
            cursor_pos: None,
            brush_size,
            grid_width: wall_rules.grid_width as i32,
            grid_height: wall_rules.grid_height as i32,
            wall_rules,
            cur_wall: None,
            walls: vec![None;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
            wall_kinds,
        }))
    }

    fn check_add_border(&self, model: &mut AreaModel, x: i32, y: i32) {
        let self_index = match self.wall_index_at(x, y) {
            Some(index) => index,
            None => return,
        };

        let tiles = self.wall_kinds[self_index].clone();
        let gh = self.grid_height;
        let gw = self.grid_width;

        let n_val = self.is_border(model, x, y, 0, -gh);
        let e_val = self.is_border(model, x, y, gw, 0);
        let s_val = self.is_border(model, x, y, 0, gh);
        let w_val = self.is_border(model, x, y, -gw, 0);
        let nw_val = self.is_border(model, x, y, -gw, -gh);
        let ne_val = self.is_border(model, x, y, gw, -gh);
        let se_val = self.is_border(model, x, y, gw, gh);
        let sw_val = self.is_border(model, x, y, -gw, gh);

        let n = self.check_index(self_index, n_val);
        let e = self.check_index(self_index, e_val);
        let s = self.check_index(self_index, s_val);
        let w = self.check_index(self_index, w_val);
        let nw = self.check_index(self_index, nw_val);
        let ne = self.check_index(self_index, ne_val);
        let se = self.check_index(self_index, se_val);
        let sw = self.check_index(self_index, sw_val);

        if n && nw && w { model.add_tile(&tiles.outer_nw, x - gw, y - gh); }

        if n && nw && ne { model.add_tile(&tiles.outer_n, x, y - gh); }

        if n && ne && e { model.add_tile(&tiles.outer_ne, x + gw, y - gh); }

        if e && ne && se { model.add_tile(&tiles.outer_e, x + gw, y); }
        else if e && ne { model.add_tile(&tiles.inner_ne, x + gw, y); }
        else if e && se { model.add_tile(&tiles.inner_se, x + gw, y); }

        if w && nw && sw { model.add_tile(&tiles.outer_w, x - gw, y); }
        else if w && nw { model.add_tile(&tiles.inner_nw, x - gw, y); }
        else if w && sw { model.add_tile(&tiles.inner_sw, x - gw, y); }

        if s && sw && se { model.add_tile(&tiles.outer_s, x, y + gh); }

        if s && se && e { model.add_tile(&tiles.outer_se, x + gw, y + gh); }

        if s && sw && w { model.add_tile(&tiles.outer_sw, x - gw, y + gh); }
    }

    fn check_index(&self, index: usize, other_index: Option<usize>) -> bool {
        match other_index {
            None => true,
            Some(other_index) => index < other_index
        }
    }

    fn is_border(&self, _model: &AreaModel, x: i32, y: i32,
                 delta_x: i32, delta_y: i32) -> Option<usize> {
        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { return None; }

        self.wall_index_at(x, y)
    }

    fn wall_index_at(&self, x: i32, y: i32) -> Option<usize> {
        self.walls[(x + y * MAX_AREA_SIZE) as usize]
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

        model.remove_tiles_within(&self.wall_rules.layer,
                                  x_min - 1 * self.grid_width,
                                  y_min - 1 * self.grid_height,
                                  (self.brush_size + 2) * self.grid_width,
                                  (self.brush_size + 2) * self.grid_height);
        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

                self.walls[(x + y * MAX_AREA_SIZE) as usize] = self.cur_wall;
            }
        }

        for yi in -2..self.brush_size+2 {
            for xi in -2..self.brush_size+2 {
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
        model.remove_tiles_within(&self.wall_rules.layer, x_min, y_min,
                                  self.brush_size * self.grid_width,
                                  self.brush_size * self.grid_height);
        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

                self.walls[(xi + yi * MAX_AREA_SIZE) as usize] = None;
            }
        }
    }
}

impl WidgetKind for WallPicker {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
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

        vec![self.brush_size_widget.clone(), brush_size_label, wall_content]
    }
}
