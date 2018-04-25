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

use rand::{self, Rng};

use sulis_core::config::CONFIG;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::{unable_to_create_error, Point};
use sulis_module::{Module, area::tile::{Tile, TerrainKind, TerrainRules}};
use sulis_widgets::{Button, Label, Spinner};

use {area_model::MAX_AREA_SIZE, AreaModel, EditorMode};

struct TerrainTiles {
    base: Rc<Tile>,
    variants: Vec<Rc<Tile>>,
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

impl TerrainTiles {
    fn new(rules: &TerrainRules, kind: TerrainKind) -> Result<TerrainTiles, Error> {
        let base_tile_id = format!("{}{}{}", rules.prefix, kind.id,
                                   rules.base_postfix);
        let base = match Module::tile(&base_tile_id) {
            None => {
                warn!("Base tile for terrain kind '{}' not found", kind.id);
                return unable_to_create_error("terrain_tiles", &kind.id);
            }, Some(tile) => tile,
        };

        let mut variants = Vec::new();
        for i in kind.variants {
            let tile_id = format!("{}{}{}{}", rules.prefix, kind.id,
                                  rules.variant_postfix, i.to_string());
            let tile = match Module::tile(&tile_id) {
                None => {
                    warn!("Tile variant '{}' not found for terrain kind '{}'", i, kind.id);
                    continue;
                }, Some(tile) => tile,
            };
            variants.push(tile);
        }

        let inner_nw = TerrainTiles::get_edge(&rules, &kind.id, &rules.inner_edge_postfix, &rules.nw_postfix)?;
        let inner_ne = TerrainTiles::get_edge(&rules, &kind.id, &rules.inner_edge_postfix, &rules.ne_postfix)?;
        let inner_sw = TerrainTiles::get_edge(&rules, &kind.id, &rules.inner_edge_postfix, &rules.sw_postfix)?;
        let inner_se = TerrainTiles::get_edge(&rules, &kind.id, &rules.inner_edge_postfix, &rules.se_postfix)?;

        let outer_n = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.n_postfix)?;
        let outer_s = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.s_postfix)?;
        let outer_e = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.e_postfix)?;
        let outer_w = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.w_postfix)?;
        let outer_ne = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.ne_postfix)?;
        let outer_nw = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.nw_postfix)?;
        let outer_se = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.se_postfix)?;
        let outer_sw = TerrainTiles::get_edge(&rules, &kind.id, &rules.outer_edge_postfix, &rules.sw_postfix)?;

        Ok(TerrainTiles {
            base,
            variants,
            inner_nw, inner_ne, inner_sw, inner_se,
            outer_n, outer_s, outer_e, outer_w, outer_se, outer_ne, outer_sw, outer_nw,
        })
    }

    fn get_edge(rules: &TerrainRules, id: &str, edge_postfix: &str, dir_postfix: &str) -> Result<Rc<Tile>, Error> {
        let tile_id = format!("{}{}{}{}", rules.prefix, id, edge_postfix, dir_postfix);

        match Module::tile(&tile_id) {
            None => {
                warn!("Edge tile with '{}', '{}' not found for '{}'", edge_postfix, dir_postfix, id);
                return unable_to_create_error("tile", &id);
            }, Some(tile) => Ok(tile),
        }
    }
}

const NAME: &str = "terrain_picker";

pub struct TerrainPicker {
    cursor_sprite: Rc<Sprite>,

    cursor_pos: Option<Point>,
    brush_size: i32,
    grid_width: i32,
    grid_height: i32,

    terrain_rules: TerrainRules,
    terrain_kinds: Vec<TerrainTiles>,

    cur_terrain: Option<usize>,

    terrain: Vec<usize>,
}

impl TerrainPicker {
    pub fn new() -> Rc<RefCell<TerrainPicker>> {
        let cursor_sprite = match ResourceSet::get_sprite(&CONFIG.editor.cursor) {
            Err(_) => panic!("Unable to find cursor sprite '{}'", CONFIG.editor.cursor),
            Ok(sprite) => sprite,
        };

        let terrain_rules = Module::terrain_rules();
        let terrain_in = Module::terrain_kinds();

        let mut terrain_kinds = Vec::new();
        for kind in terrain_in {
            match TerrainTiles::new(&terrain_rules, kind) {
                Err(_) => continue,
                Ok(terrain) => terrain_kinds.push(terrain),
            }
        }

        Rc::new(RefCell::new(TerrainPicker {
            cursor_sprite,
            cursor_pos: None,
            brush_size: 4,
            grid_width: terrain_rules.grid_width as i32,
            grid_height: terrain_rules.grid_height as i32,
            terrain_rules,
            terrain_kinds,
            cur_terrain: None,
            terrain: vec![0;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
        }))
    }

    fn gen_choice(&self, tiles: &TerrainTiles) -> Rc<Tile> {
        if tiles.variants.len() == 0 { return Rc::clone(&tiles.base); }

        let base_weight = self.terrain_rules.base_weight;
        let total_weight = base_weight + tiles.variants.len() as u32;

        let roll = rand::thread_rng().gen_range(0, total_weight);

        if roll < base_weight {
            return Rc::clone(&tiles.base);
        }

        return Rc::clone(&tiles.variants[(roll - base_weight) as usize]);
    }

    fn check_add_border(&self, tiles: &TerrainTiles, model: &mut AreaModel, x: i32, y: i32) {
        let self_index = self.terrain[(x + y * MAX_AREA_SIZE) as usize];

        let gh = self.grid_height;
        let gw = self.grid_width;

        let n = self.is_border(self_index, x, y, 0, -gh);
        let e = self.is_border(self_index, x, y, gw, 0);
        let s = self.is_border(self_index, x, y, 0, gh);
        let w = self.is_border(self_index, x, y, -gw, 0);
        let nw = self.is_border(self_index, x, y, -gw, -gh);
        let ne = self.is_border(self_index, x, y, gw, -gh);
        let se = self.is_border(self_index, x, y, gw, gh);
        let sw = self.is_border(self_index, x, y, -gw, gh);

        if n && nw && w { model.add_tile(Rc::clone(&tiles.outer_nw), x - gw, y - gh); }

        if n && nw && ne { model.add_tile(Rc::clone(&tiles.outer_n), x, y - gh); }


        if n && ne && e { model.add_tile(Rc::clone(&tiles.outer_ne), x + gw, y - gh); }

        if e && ne && se { model.add_tile(Rc::clone(&tiles.outer_e), x + gw, y); }
        else if e && ne { model.add_tile(Rc::clone(&tiles.inner_ne), x + gw, y); }
        else if e && se { model.add_tile(Rc::clone(&tiles.inner_se), x + gw, y); }

        if w && nw && sw { model.add_tile(Rc::clone(&tiles.outer_w), x - gw, y); }
        else if w && nw { model.add_tile(Rc::clone(&tiles.inner_nw), x - gw, y); }
        else if w && sw { model.add_tile(Rc::clone(&tiles.inner_sw), x - gw, y); }

        if s && sw && se { model.add_tile(Rc::clone(&tiles.outer_s), x, y + gh); }


        if s && se && e { model.add_tile(Rc::clone(&tiles.outer_se), x + gw, y + gh); }

        if s && sw && w { model.add_tile(Rc::clone(&tiles.outer_sw), x - gw, y + gh); }
    }

    fn is_border(&self, self_index: usize, x: i32, y: i32, delta_x: i32, delta_y: i32) -> bool {
        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { return false; }

        self_index > self.terrain[(x + y * MAX_AREA_SIZE) as usize]
    }
}

impl EditorMode for TerrainPicker {
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

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let (index, cur_terrain) = match self.cur_terrain {
            None => return,
            Some(i) => (i, &self.terrain_kinds[i]),
        };

        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_tiles_within(&self.terrain_rules.border_layer,
                                  x_min - self.grid_width, y_min - self.grid_height,
                                  (self.brush_size + 2) * self.grid_width,
                                  (self.brush_size + 2) * self.grid_height);
        model.remove_tiles_within(&self.terrain_rules.base_layer,
                                  x_min, y_min,
                                  self.brush_size * self.grid_width,
                                  self.brush_size * self.grid_height);

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

                self.terrain[(x + y * MAX_AREA_SIZE) as usize] = index;

                model.add_tile(self.gen_choice(cur_terrain), x, y);
            }
        }

        for yi in -1..self.brush_size+1 {
            for xi in -1..self.brush_size+1 {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }
                self.check_add_border(cur_terrain, model, x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_all_tiles(x_min, y_min, self.brush_size * self.grid_width,
                                self.brush_size * self.grid_height);
    }
}

impl WidgetKind for TerrainPicker {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let brush_size = Widget::with_theme(Spinner::new(self.brush_size, 1, 20), "brush_size");
        brush_size.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.brush_size = spinner.value();
        })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let terrain_content = Widget::empty("terrain_content");
        for (i, terrain_kind) in self.terrain_kinds.iter().enumerate() {
            let button = Widget::with_theme(Button::empty(), "terrain_button");
            button.borrow_mut().state.add_text_arg("icon", &terrain_kind.base.image_display.full_id());

            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let parent = Widget::get_parent(&parent);
                    let terrain_picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);
                    terrain_picker.cur_terrain = Some(i);
                }
            }));
            button.borrow_mut().state.add_callback(cb);

            Widget::add_child_to(&terrain_content, button);
        }

        vec![brush_size, brush_size_label, terrain_content]
    }
}
