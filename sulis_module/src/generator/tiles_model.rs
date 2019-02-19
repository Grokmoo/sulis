//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::rc::Rc;

use sulis_core::config::Config;
use sulis_core::util::{gen_rand, Point};
use crate::Module;
use crate::area::{MAX_AREA_SIZE, Tile};
use crate::generator::{TerrainTiles, WallTiles};

pub struct TilesModel {
    pub grid_width: i32,
    pub grid_height: i32,

    tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)>,

    elevation: Vec<u8>,

    terrain_kinds: Vec<TerrainTiles>,
    terrain: Vec<Option<usize>>,

    wall_kinds: Vec<WallTiles>,
    walls: Vec<(u8, Option<usize>)>,
}

impl TilesModel {
    pub fn new() -> TilesModel {
        let terrain_rules = Module::terrain_rules();
        let terrain_kinds = Module::terrain_kinds();
        let mut terrain_out = Vec::new();
        for kind in terrain_kinds.iter() {
            match TerrainTiles::new(&terrain_rules, kind, &terrain_kinds) {
                Err(_) => continue,
                Ok(terrain) => terrain_out.push(terrain),
            }
        }

        let wall_rules = Module::wall_rules();
        let mut walls_out = Vec::new();
        for kind in Module::wall_kinds() {
            match WallTiles::new(kind, &wall_rules) {
                Err(_) => continue,
                Ok(walls) => walls_out.push(walls),
            }
        }

        let mut tiles = Vec::new();
        for layer_id in Config::editor_config().area.layers {
            tiles.push((layer_id, Vec::new()));
        }

        TilesModel {
            grid_width: terrain_rules.grid_width as i32,
            grid_height: terrain_rules.grid_height as i32,
            tiles,
            elevation: vec![0; (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
            terrain_kinds: terrain_out,
            terrain: vec![None; (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
            wall_kinds: walls_out,
            walls: vec![(0, None); (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, Vec<(Point, Rc<Tile>)>)> {
        self.tiles.iter()
    }

    pub fn clear(&mut self) {
        for (_, layer) in self.tiles.iter_mut() {
            layer.clear();
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &(Point, Rc<Tile>)> {
        self.tiles
            .iter()
            .map(|ref v| &v.1)
            .flat_map(|ref t| t.iter())
    }

    pub fn add_some(&mut self, tile: &Option<Rc<Tile>>, x: i32, y: i32) {
        if let Some(tile) = tile {
            self.add(Rc::clone(tile), x, y);
        }
    }

    pub fn add(&mut self, tile: Rc<Tile>, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let index = self.create_layer_if_missing(&tile.layer);

        // check if the tile already exists
        for &(p, ref other_tile) in self.tiles[index].1.iter() {
            if p.x == x && p.y == y && Rc::ptr_eq(other_tile, &tile) {
                return;
            }
        }
        self.tiles[index].1.push((Point::new(x, y), tile));
    }

    pub fn remove_all(&mut self, x: i32, y: i32, width: i32, height: i32) {
        for &mut (_, ref mut tiles) in self.tiles.iter_mut() {
            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile.width, tile.height, x, y, width, height)
            });
        }
    }

    pub fn remove_within(&mut self, layer_id: &str, x: i32, y: i32, width: i32, height: i32) {
        for &mut (ref cur_layer_id, ref mut tiles) in self.tiles.iter_mut() {
            if layer_id != cur_layer_id {
                continue;
            }

            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile.width, tile.height, x, y, width, height)
            });
        }
    }

    pub fn within(
        &self,
        layer_id: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<(Point, Rc<Tile>)> {
        let mut within = Vec::new();

        for &(ref cur_layer_id, ref tiles) in self.tiles.iter() {
            if layer_id != cur_layer_id {
                continue;
            }

            for &(pos, ref tile) in tiles {
                if !is_removal(pos, tile.width, tile.height, x, y, width, height) {
                    continue;
                }

                within.push((pos, Rc::clone(tile)));
            }
        }

        within
    }

    pub fn shift(&mut self, delta_x: i32, delta_y: i32) {
        for &mut (_, ref mut layer) in self.tiles.iter_mut() {
            for &mut (ref mut point, ref tile) in layer.iter_mut() {
                if point.x + delta_x < 0
                    || point.y + delta_y < 0
                    || point.x + delta_x + tile.width > MAX_AREA_SIZE
                    || point.y + delta_y + tile.height > MAX_AREA_SIZE
                {
                    warn!("Invalid tile shift parameters: {},{}", delta_x, delta_y);
                    return;
                }
            }
        }

        for &mut (_, ref mut layer) in self.tiles.iter_mut() {
            for &mut (ref mut point, _) in layer.iter_mut() {
                point.x += delta_x;
                point.y += delta_y;
            }
        }
    }

    fn create_layer_if_missing(&mut self, layer_id: &str) -> usize {
        for (index, &(ref id, _)) in self.tiles.iter().enumerate() {
            if id == layer_id {
                return index;
            }
        }

        self.tiles.push((layer_id.to_string(), Vec::new()));
        self.tiles.len() - 1
    }

    pub fn elevation(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 {
            return 0;
        }

        let index = (x + y * MAX_AREA_SIZE) as usize;
        if index >= (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize {
            return 0;
        }

        self.elevation[index]
    }

    pub fn set_elevation(&mut self, elev: u8, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let index = (x + y * MAX_AREA_SIZE) as usize;
        if index >= (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize {
            return;
        }

        self.elevation[index] = elev;
    }

    pub fn wall_kind(&self, index: usize) -> &WallTiles {
        &self.wall_kinds[index]
    }

    pub fn wall_kinds(&self) -> &[WallTiles] {
        &self.wall_kinds
    }

    pub fn wall_at(&self, x: i32, y: i32) -> (u8, Option<usize>) {
        self.walls[(x + y * MAX_AREA_SIZE) as usize]
    }

    pub fn set_wall(&mut self, x: i32, y: i32, elev: u8, index: Option<usize>) {
        self.walls[(x + y * MAX_AREA_SIZE) as usize] = (elev, index);
    }

    pub fn terrain_index_at(&self, x: i32, y: i32) -> Option<usize> {
        self.terrain[(x + y * MAX_AREA_SIZE) as usize]
    }

    pub fn terrain_kind(&self, index: usize) -> &TerrainTiles {
        &self.terrain_kinds[index]
    }

    pub fn terrain_kinds(&self) -> &[TerrainTiles] {
        &self.terrain_kinds
    }

    pub fn set_terrain_index(&mut self, x: i32, y: i32, index: Option<usize>) {
        self.terrain[(x + y * MAX_AREA_SIZE) as usize] = index;
    }

    pub fn raw_elevation(&mut self) -> &mut [u8] {
        &mut self.elevation
    }

    pub fn gen_choice(&self, tiles: &TerrainTiles) -> Rc<Tile> {
        if tiles.variants.len() == 0 {
            return Rc::clone(&tiles.base);
        }

        let base_weight = tiles.base_weight;
        let total_weight = base_weight + tiles.variants.len() as u32;

        let roll = gen_rand(0, total_weight);

        if roll < base_weight {
            return Rc::clone(&tiles.base);
        }

        return Rc::clone(&tiles.variants[(roll - base_weight) as usize]);
    }

    pub fn check_add_terrain_border(&mut self, x: i32, y: i32) {
        let self_index = match self.terrain_index_at(x, y) {
            Some(index) => index,
            None => return,
        };
        let tiles = self.terrain_kind(self_index).clone();
        let gh = self.grid_height;
        let gw = self.grid_width;

        let n_val = self.is_terrain_border(x, y, 0, -gh);
        let e_val = self.is_terrain_border(x, y, gw, 0);
        let s_val = self.is_terrain_border(x, y, 0, gh);
        let w_val = self.is_terrain_border(x, y, -gw, 0);
        let nw_val = self.is_terrain_border(x, y, -gw, -gh);
        let ne_val = self.is_terrain_border(x, y, gw, -gh);
        let se_val = self.is_terrain_border(x, y, gw, gh);
        let sw_val = self.is_terrain_border(x, y, -gw, gh);

        let n = check_index(self_index, n_val);
        let e = check_index(self_index, e_val);
        let s = check_index(self_index, s_val);
        let w = check_index(self_index, w_val);
        let nw = check_index(self_index, nw_val);
        let ne = check_index(self_index, ne_val);
        let se = check_index(self_index, se_val);
        let sw = check_index(self_index, sw_val);

        if n && nw && w {
            self.add_some(&tiles.matching_edges(nw_val).outer_nw, x - gw, y - gh);
        }

        if n && nw && ne {
            self.add_some(&tiles.matching_edges(n_val).outer_n, x, y - gh);
        }

        if n && ne && e {
            self.add_some(&tiles.matching_edges(ne_val).outer_ne, x + gw, y - gh);
        }

        if e && ne && se {
            self.add_some(&tiles.matching_edges(e_val).outer_e, x + gw, y);
        } else if e && ne {
            self.add_some(&tiles.matching_edges(ne_val).inner_ne, x + gw, y);
        } else if e && se {
            self.add_some(&tiles.matching_edges(se_val).inner_se, x + gw, y);
        }

        if w && nw && sw {
            self.add_some(&tiles.matching_edges(w_val).outer_w, x - gw, y);
        } else if w && nw {
            self.add_some(&tiles.matching_edges(nw_val).inner_nw, x - gw, y);
        } else if w && sw {
            self.add_some(&tiles.matching_edges(sw_val).inner_sw, x - gw, y);
        }

        if s && sw && se {
            self.add_some(&tiles.matching_edges(s_val).outer_s, x, y + gh);
        }

        if s && se && e {
            self.add_some(&tiles.matching_edges(se_val).outer_se, x + gw, y + gh);
        }

        if s && sw && w {
            self.add_some(&tiles.matching_edges(sw_val).outer_sw, x - gw, y + gh);
        }
    }

    fn is_terrain_border(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> Option<usize> {
        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
            return None;
        }

        self.terrain_index_at(x, y)
    }

    pub fn check_add_wall_border(&mut self, x: i32, y: i32) {
        let (self_elev, self_index) = match self.wall_at(x, y) {
            (elev, Some(index)) => (elev, index),
            (_, None) => return,
        };

        let tiles = self.wall_kind(self_index).clone();

        if tiles.interior_border {
            self.check_add_wall_border_interior(x, y, self_elev, tiles);
        } else {
            self.check_add_wall_border_exterior(x, y, self_elev, tiles);
        }
    }

    fn check_add_wall_border_exterior(&mut self, x: i32, y: i32, self_elev: u8, tiles: WallTiles) {
        self.add_some(&tiles.fill_tile, x, y);

        let (gh, gw) = (self.grid_height, self.grid_width);

        let n = self.is_wall_border(self_elev, x, y, 0, -gh);
        let e = self.is_wall_border(self_elev, x, y, gw, 0);
        let s = self.is_wall_border(self_elev, x, y, 0, gh);
        let w = self.is_wall_border(self_elev, x, y, -gw, 0);
        let nw = self.is_wall_border(self_elev, x, y, -gw, -gh);
        let ne = self.is_wall_border(self_elev, x, y, gw, -gh);
        let se = self.is_wall_border(self_elev, x, y, gw, gh);
        let sw = self.is_wall_border(self_elev, x, y, -gw, gh);

        if n && nw && w {
            self.add_some(&tiles.edges.outer_nw, x - gw, y - gh);
        }

        if n && nw && ne {
            self.add_some(&tiles.edges.outer_n, x, y - gh);
        }

        if n && ne && e {
            self.add_some(&tiles.edges.outer_ne, x + gw, y - gh);
        }

        if e && ne && se {
            self.add_some(&tiles.edges.outer_e, x + gw, y);
        } else if e && ne {
            self.add_some(&tiles.edges.inner_ne, x + gw, y);
        } else if e && se {
            self.add_some(&tiles.edges.inner_se, x + gw, y);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                self.add_some(&ext.outer_s, x + gw, y + offset * gh);
            }
        }

        if w && nw && sw {
            self.add_some(&tiles.edges.outer_w, x - gw, y);
        } else if w && nw {
            self.add_some(&tiles.edges.inner_nw, x - gw, y);
        } else if w && sw {
            self.add_some(&tiles.edges.inner_sw, x - gw, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                self.add_some(&ext.outer_s, x - gw, y + offset * gh);
            }
        }

        if s && sw && se {
            self.add_some(&tiles.edges.outer_s, x, y + gh);
            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                self.add_some(&ext.outer_s, x, y + offset * gh);
            }
        }

        if s && se && e {
            self.add_some(&tiles.edges.outer_se, x + gw, y + gh);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                self.add_some(&ext.outer_se, x + gw, y + offset * gh);
            }
        }

        if s && sw && w {
            self.add_some(&tiles.edges.outer_sw, x - gw, y + gh);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 2 + i as i32;
                self.add_some(&ext.outer_sw, x - gw, y + offset * gh);
            }
        }
    }

    fn check_add_wall_border_interior(&mut self, x: i32, y: i32, self_elev: u8, tiles: WallTiles) {
        let (gh, gw) = (self.grid_height, self.grid_width);

        let n = self.is_wall_border(self_elev, x, y, 0, -gh);
        let e = self.is_wall_border(self_elev, x, y, gw, 0);
        let s = self.is_wall_border(self_elev, x, y, 0, gh);
        let w = self.is_wall_border(self_elev, x, y, -gw, 0);
        let nw = self.is_wall_border(self_elev, x, y, -gw, -gh);
        let ne = self.is_wall_border(self_elev, x, y, gw, -gh);
        let se = self.is_wall_border(self_elev, x, y, gw, gh);
        let sw = self.is_wall_border(self_elev, x, y, -gw, gh);

        if !n && !e && !s && !w && !nw && !ne && !se && !sw {
            self.add_some(&tiles.fill_tile, x, y);
            return;
        }

        if n && e && s && w && nw && ne && se && sw {
            self.add_some(&tiles.edges.outer_all, x, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                self.add_some(&ext.outer_all, x, y + (i as i32 + 1) * gh);
            }
            return;
        }

        if ne && sw && !n && !s && !e && !w {
            self.add_some(&tiles.edges.inner_ne_sw, x, y);
            return;
        }

        if nw && se && !n && !s && !e && !w {
            self.add_some(&tiles.edges.inner_nw_se, x, y);
            return;
        }

        if n && nw && w {
            self.add_some(&tiles.edges.outer_nw, x, y);
        }

        if n && !w && !e {
            self.add_some(&tiles.edges.outer_n, x, y);
        }

        if n && ne && e {
            self.add_some(&tiles.edges.outer_ne, x, y);
        }

        if e && !n && !s {
            self.add_some(&tiles.edges.outer_e, x, y);
        }
        if w && !n && !s {
            self.add_some(&tiles.edges.outer_w, x, y);
        }

        if nw && !n && !w {
            self.add_some(&tiles.edges.inner_nw, x, y);
        }
        if ne && !n && !e {
            self.add_some(&tiles.edges.inner_ne, x, y);
        }

        if sw && !s && !w {
            self.add_some(&tiles.edges.inner_sw, x, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                self.add_some(&ext.outer_s, x, y + (i as i32 + 1) * gh);
            }
        }

        if se && !s && !e {
            self.add_some(&tiles.edges.inner_se, x, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                self.add_some(&ext.outer_s, x, y + (i as i32 + 1) * gh);
            }
        }

        if s && !w && !e {
            self.add_some(&tiles.edges.outer_s, x, y);
            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                self.add_some(&ext.outer_s, x, y + offset * gh);
            }
        }

        if s && se && e {
            self.add_some(&tiles.edges.outer_se, x, y);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                self.add_some(&ext.outer_se, x, y + offset * gh);
            }
        }

        if s && sw && w {
            self.add_some(&tiles.edges.outer_sw, x, y);

            for (i, ext) in tiles.extended.iter().enumerate() {
                let offset = 1 + i as i32;
                self.add_some(&ext.outer_sw, x, y + offset * gh);
            }
        }
    }

    fn is_wall_border(
        &self,
        self_elev: u8,
        x: i32,
        y: i32,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        let x = x + delta_x;
        let y = y + delta_y;
        if x < 0 || x >= MAX_AREA_SIZE || y < 0 || y >= MAX_AREA_SIZE {
            return false;
        }

        let (dest_elev, dest_index) = self.wall_at(x, y);

        match dest_index {
            None => true,
            Some(_) => dest_elev < self_elev,
        }
    }
}

fn check_index(index: usize, other_index: Option<usize>) -> bool {
    match other_index {
        None => true,
        Some(other_index) => index < other_index,
    }
}

pub fn is_removal(pos: Point, pos_w: i32, pos_h: i32, x: i32, y: i32, w: i32, h: i32) -> bool {
    // if one rectangle is on left side of the other
    if pos.x >= x + w || x >= pos.x + pos_w {
        return false;
    }

    // if one rectangle is above the other
    if pos.y >= y + h || y >= pos.y + pos_h {
        return false;
    }

    true
}
