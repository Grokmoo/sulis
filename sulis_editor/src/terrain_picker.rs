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

use std::collections::HashMap;
use std::io::Error;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use rand::{self, Rng};

use sulis_core::config::Config;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::{unable_to_create_error, Point};
use sulis_module::{Module, area::MAX_AREA_SIZE, area::tile::{Tile, TerrainKind, TerrainRules, EdgeRules}};
use sulis_widgets::{Button, Label, Spinner, ScrollPane};

use crate::{AreaModel, EditorMode};

#[derive(Clone)]
pub struct TerrainTiles {
    pub id: String,
    base: Rc<Tile>,
    base_weight: u32,
    variants: Vec<Rc<Tile>>,
    edges: EdgesList,
    borders: HashMap<usize, EdgesList>,
}

#[derive(Clone)]
pub struct EdgesList {
    pub inner_nw: Option<Rc<Tile>>,
    pub inner_ne: Option<Rc<Tile>>,
    pub inner_sw: Option<Rc<Tile>>,
    pub inner_se: Option<Rc<Tile>>,

    pub outer_n: Option<Rc<Tile>>,
    pub outer_s: Option<Rc<Tile>>,
    pub outer_e: Option<Rc<Tile>>,
    pub outer_w: Option<Rc<Tile>>,
    pub outer_se: Option<Rc<Tile>>,
    pub outer_ne: Option<Rc<Tile>>,
    pub outer_sw: Option<Rc<Tile>>,
    pub outer_nw: Option<Rc<Tile>>,

    pub outer_all: Option<Rc<Tile>>,
    pub inner_ne_sw: Option<Rc<Tile>>,
    pub inner_nw_se: Option<Rc<Tile>>,
}

impl EdgesList {
    pub fn new(id: &str, prefix: &str, rules: &EdgeRules) -> Result<EdgesList, Error> {
        let inner_nw = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.nw_postfix);
        let inner_ne = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.ne_postfix);
        let inner_sw = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.sw_postfix);
        let inner_se = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.se_postfix);

        let outer_n = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.n_postfix);
        let outer_s = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.s_postfix);
        let outer_e = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.e_postfix);
        let outer_w = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.w_postfix);
        let outer_ne = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.ne_postfix);
        let outer_nw = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.nw_postfix);
        let outer_se = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.se_postfix);
        let outer_sw = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.sw_postfix);
        let outer_all = EdgesList::get_edge(&prefix, &id, &rules.outer_edge_postfix, &rules.all_postfix);

        let inner_ne_sw = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.ne_sw_postfix);

        let inner_nw_se = EdgesList::get_edge(&prefix, &id, &rules.inner_edge_postfix, &rules.nw_se_postfix);

        Ok(EdgesList {
            inner_nw, inner_ne, inner_sw, inner_se,
            outer_n, outer_s, outer_e, outer_w, outer_se, outer_ne, outer_sw, outer_nw,
            outer_all, inner_ne_sw, inner_nw_se,
        })
    }

    fn get_edge(prefix: &str, id: &str, edge_postfix: &str, dir_postfix: &str) -> Option<Rc<Tile>> {
        let tile_id = format!("{}{}{}{}", prefix, id, edge_postfix, dir_postfix);

        match Module::tile(&tile_id) {
            None => {
                trace!("Edge tile with '{}', '{}' not found for '{}'", edge_postfix, dir_postfix, id);
                None
            }, Some(tile) => Some(tile),
        }
    }
}

impl TerrainTiles {
    pub fn new(rules: &TerrainRules, kind: TerrainKind,
               all_kinds: &Vec<TerrainKind>) -> Result<TerrainTiles, Error> {
        let base_tile_id = format!("{}{}{}", rules.prefix, kind.id,
                                   rules.base_postfix);
        let base = match Module::tile(&base_tile_id) {
            None => {
                warn!("Base tile for terrain kind '{}' not found", kind.id);
                return unable_to_create_error("terrain_tiles", &kind.id);
            }, Some(tile) => tile,
        };

        let base_weight = match kind.base_weight {
            None => rules.base_weight,
            Some(weight) => weight,
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

        let mut borders = HashMap::new();
        for (other_terrain, id) in kind.borders.iter() {
            let edges = EdgesList::new(id, &rules.prefix, &rules.edges)?;

            let mut index = None;
            for (i, other_kind) in all_kinds.iter().enumerate() {
                if &other_kind.id == other_terrain {
                    index = Some(i);
                    break;
                }
            }

            match index {
                None => {
                    warn!("Other terrain '{}' not found for border of '{}'", other_terrain, kind.id);
                    continue;
                }, Some(index) => {
                    borders.insert(index, edges);
                }
            }
        }

        let edges = EdgesList::new(&kind.id, &rules.prefix, &rules.edges)?;

        Ok(TerrainTiles {
            id: kind.id,
            base,
            base_weight,
            variants,
            borders,
            edges,
        })
    }

    fn matching_edges(&self, index: Option<usize>) -> &EdgesList {
        match index {
            None => &self.edges,
            Some(index) => {
                match self.borders.get(&index) {
                    None => &self.edges,
                    Some(ref edges) => edges,
                }
            }
        }
    }
}

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
        let cursor_sprite = match ResourceSet::get_sprite(&Config::editor_config().cursor) {
            Err(_) => panic!("Unable to find cursor sprite '{}'", Config::editor_config().cursor),
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

    fn gen_choice(&self, tiles: &TerrainTiles) -> Rc<Tile> {
        if tiles.variants.len() == 0 { return Rc::clone(&tiles.base); }

        let base_weight = tiles.base_weight;
        let total_weight = base_weight + tiles.variants.len() as u32;

        let roll = rand::thread_rng().gen_range(0, total_weight);

        if roll < base_weight {
            return Rc::clone(&tiles.base);
        }

        return Rc::clone(&tiles.variants[(roll - base_weight) as usize]);
    }

    fn check_add_border(&self, model: &mut AreaModel, x: i32, y: i32) {
        let self_index = match model.terrain_index_at(x, y) {
            Some(index) => index,
            None => return,
        };
        let tiles = model.terrain_kind(self_index).clone();
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

        if n && nw && w { model.add_tile(&tiles.matching_edges(nw_val).outer_nw, x - gw, y - gh); }

        if n && nw && ne { model.add_tile(&tiles.matching_edges(n_val).outer_n, x, y - gh); }

        if n && ne && e { model.add_tile(&tiles.matching_edges(ne_val).outer_ne, x + gw, y - gh); }

        if e && ne && se { model.add_tile(&tiles.matching_edges(e_val).outer_e, x + gw, y); }
        else if e && ne { model.add_tile(&tiles.matching_edges(ne_val).inner_ne, x + gw, y); }
        else if e && se { model.add_tile(&tiles.matching_edges(se_val).inner_se, x + gw, y); }

        if w && nw && sw { model.add_tile(&tiles.matching_edges(w_val).outer_w, x - gw, y); }
        else if w && nw { model.add_tile(&tiles.matching_edges(nw_val).inner_nw, x - gw, y); }
        else if w && sw { model.add_tile(&tiles.matching_edges(sw_val).inner_sw, x - gw, y); }

        if s && sw && se { model.add_tile(&tiles.matching_edges(s_val).outer_s, x, y + gh); }

        if s && se && e { model.add_tile(&tiles.matching_edges(se_val).outer_se, x + gw, y + gh); }

        if s && sw && w { model.add_tile(&tiles.matching_edges(sw_val).outer_sw, x - gw, y + gh); }
    }

    fn check_index(&self, index: usize, other_index: Option<usize>) -> bool {
        match other_index {
            None => true,
            Some(other_index) => index < other_index
        }
    }

    fn is_border(&self, model: &AreaModel, x: i32, y: i32,
                 delta_x: i32, delta_y: i32) -> Option<usize> {
        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { return None; }

        model.terrain_index_at(x, y)
    }
}

impl EditorMode for TerrainPicker {
    fn draw_mode(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x_offset: f32, y_offset: f32,
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
        let (index, cur_terrain) = match self.cur_terrain {
            None => return,
            Some(i) => (i, model.terrain_kind(i).clone()),
        };

        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_tiles_within(&self.terrain_rules.border_layer,
                                  x_min - 1 * self.grid_width, y_min - 1 * self.grid_height,
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

                model.set_terrain_index(x, y, Some(index));
                model.add_tile(&Some(self.gen_choice(&cur_terrain)), x, y);
            }
        }

        // add tiles in a larger radius than we removed -
        // we rely on the model to not add already existing tiles twice
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
        model.remove_all_tiles(x_min, y_min, self.brush_size * self.grid_width,
                                self.brush_size * self.grid_height);

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;
                if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE { continue; }

                model.set_terrain_index(x, y, None);
            }
        }
    }
}

impl WidgetKind for TerrainPicker {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.brush_size_widget.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.brush_size = spinner.value();
        })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let scrollpane = ScrollPane::new();
        for (i, terrain_kind) in Module::terrain_kinds().into_iter().enumerate() {
            let base_tile_id = format!("{}{}{}", self.terrain_rules.prefix, terrain_kind.id,
                                       self.terrain_rules.base_postfix);

            let button = Widget::with_theme(Button::empty(), "terrain_button");
            button.borrow_mut().state.add_text_arg("icon", &base_tile_id);

            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let parent = Widget::go_up_tree(&parent, 2);
                    let terrain_picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);
                    terrain_picker.cur_terrain = Some(i);
                }
            }));
            button.borrow_mut().state.add_callback(cb);

            scrollpane.borrow().add_to_content(button);
        }

        vec![self.brush_size_widget.clone(), brush_size_label, Widget::with_theme(scrollpane, "terrain")]
    }
}
