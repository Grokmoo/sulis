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

mod area_generator;
pub use self::area_generator::AreaGenerator;

mod encounter_gen;
use self::encounter_gen::{EncounterGen, EncounterParams, EncounterParamsBuilder};

mod feature_gen;
use self::feature_gen::{FeatureGen, FeatureParams, FeatureParamsBuilder};

mod maze;
use self::maze::{Maze, TileKind};

mod prop_gen;
use self::prop_gen::{PropGen, PropParams, PropParamsBuilder};

mod terrain_gen;
use self::terrain_gen::{TerrainGen, TerrainParams, TerrainParamsBuilder};

mod terrain_tiles;
pub use self::terrain_tiles::{EdgesList, TerrainTiles};

mod tiles_model;
pub use self::tiles_model::{TilesModel, is_removal};

mod transition_gen;
use self::transition_gen::{TransitionGen, TransitionParams, TransitionParamsBuilder};

mod wall_tiles;
pub use self::wall_tiles::{WallTiles};

use std::collections::{HashMap};
use std::io::{Error, ErrorKind};

use sulis_core::util::{Point, gen_rand};
use crate::area::{AreaBuilder, Layer, PropDataBuilder, EncounterDataBuilder, TransitionBuilder};
use crate::{WallKind};

pub struct WeightedList<T> {
    total_weight: u32,
    entries: Vec<(T, u32)>,
}

impl<T> WeightedList<T> {
    pub fn new<F>(kinds: HashMap<String, WeightedEntry>, name: &str, getter: F)
        -> Result<WeightedList<T>, Error> where F: Fn(&str) -> Option<T> {
        let mut entries = Vec::new();
        let mut total_weight = 0;
        for (id, entry) in kinds {
            let t = getter(&id).ok_or(
                Error::new(ErrorKind::InvalidInput, format!("Invalid {} '{}'", name, id))
            )?;

            total_weight += entry.weight;
            entries.push((t, entry.weight));
        }

        if total_weight == 0 || entries.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidInput,
                format!("Must specify at least one {}", name)));
        }

        Ok(WeightedList {
            entries,
            total_weight,
        })
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    pub fn pick(&self) -> &T {
        if self.entries.len() == 1 || self.total_weight == 1 {
            return &self.entries[0].0;
        }

        let roll = gen_rand(0, self.total_weight);

        let mut cur_weight = 0;
        for (kind, weight) in self.entries.iter() {
            cur_weight += weight;
            if roll < cur_weight {
                return kind;
            }
        }

        unreachable!()
    }
}

struct WallKinds {
    kinds: WeightedList<WallKind>,
}

impl WallKinds {
    fn pick_index(&self, model: &TilesModel) -> Option<usize> {
        let wall_kind = self.kinds.pick();
        let mut wall_index = None;
        for (index, kind) in model.wall_kinds().iter().enumerate() {
            if kind.id == wall_kind.id {
                wall_index = Some(index);
                break;
            }
        }

        if let None = wall_index {
            error!("Invalid wall kind '{}'.  This is a bug", wall_kind.id);
            panic!();
        }

        wall_index
    }
}

pub struct GeneratorOutput {
    pub layers: Vec<Layer>,
    pub props: Vec<PropDataBuilder>,
    pub encounters: Vec<EncounterDataBuilder>,
    pub transitions: Vec<TransitionBuilder>,
}

pub(crate) struct GenModel<'a> {
    model: TilesModel,
    builder: &'a AreaBuilder,
    total_grid_size: Point,
    region_overfill_edges: HashMap<usize, usize>,
}

impl<'a> GenModel<'a> {
    fn new(builder: &'a AreaBuilder, gen_grid_width: i32, gen_grid_height: i32) -> GenModel<'a> {
        let model = TilesModel::new();
        let gen_grid_size = Point::new(gen_grid_width, gen_grid_height);
        let total_grid_size = Point::new(gen_grid_size.x * model.grid_width,
                                         gen_grid_size.y * model.grid_height);
        GenModel {
            model,
            builder,
            total_grid_size,
            region_overfill_edges: HashMap::new(),
        }
    }

    fn region_size(&self) -> (u32, u32) {
        let x = (self.builder.width as i32 - 2 * self.model.grid_width)
            / self.total_grid_size.x;
        let y = (self.builder.height as i32 - 2 * self.model.grid_height)
            / self.total_grid_size.y;
        (x as u32, y as u32)
    }

    fn to_region_coords(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.model.grid_width) / self.total_grid_size.x,
         (y - self.model.grid_height) / self.total_grid_size.y)
    }

    fn from_region_coords(&self, x: i32, y: i32) -> (i32, i32) {
        (x * self.total_grid_size.x + self.model.grid_width,
         y * self.total_grid_size.y + self.model.grid_height)
    }

    fn tiles(&self) -> TileIter {
        TileIter::new(self)
    }
}

pub struct TileIter {
    max_x: i32,
    max_y: i32,
    step_x: i32,
    step_y: i32,
    x: i32,
    y: i32,
}

impl TileIter {
    fn simple(max_x: i32, max_y: i32) -> TileIter {
        TileIter {
            x: 0,
            y: 0,
            max_x,
            max_y,
            step_x: 1,
            step_y: 1,
        }
    }

    fn new<'a>(model: &'a GenModel<'a>) -> TileIter {
        TileIter {
            x: 0,
            y: 0,
            max_x: model.builder.width as i32,
            max_y: model.builder.height as i32,
            step_x: model.model.grid_width as i32,
            step_y: model.model.grid_height as i32,
        }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

impl Iterator for TileIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.y >= self.max_y { return None; }

        let ret_val = Some(Point::new(self.x, self.y));

        self.x += self.step_x;

        if self.x >= self.max_x {
            self.x = 0;
            self.y += self.step_y;
        }

        ret_val
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorBuilder {
    id: String,
    wall_kinds: HashMap<String, WeightedEntry>,
    grid_width: u32,
    grid_height: u32,
    rooms: RoomParams,
    terrain: TerrainParamsBuilder,
    props: PropParamsBuilder,
    encounters: EncounterParamsBuilder,
    features: FeatureParamsBuilder,
    transitions: TransitionParamsBuilder,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedEntry {
    weight: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoomParams {
    min_size: Point,
    max_size: Point,
    min_spacing: u32,
    room_placement_attempts: u32,
    winding_chance: u32,
    extra_connection_chance: u32,
    dead_end_keep_chance: u32,
    invert: bool,
    gen_corridors: bool,
    room_edge_overfill_chance: u32,
    corridor_edge_overfill_chance: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum RegionKind {
    Wall,
    Corridor,
    Room,
    TransitionRoom,
    Doorway,
}

pub struct RegionKinds {
    allowed: [bool; 5],
}

impl RegionKinds {
    pub fn new(src: Vec<RegionKind>) -> RegionKinds {
        let allowed = [
            src.contains(&RegionKind::Wall),
            src.contains(&RegionKind::Corridor),
            src.contains(&RegionKind::Room),
            src.contains(&RegionKind::TransitionRoom),
            src.contains(&RegionKind::Doorway),
        ];

        RegionKinds {
            allowed
        }
    }

    pub(crate) fn check_coords(&self, maze: &Maze, p1: Point, p2: Point) -> bool {
        for y in p1.y..=p2.y {
            for x in p1.x..=p2.x {
                let t = maze.tile_checked(x, y);
                if !self.is_allowable(t) { return false; }
            }
        }

        true
    }

    pub fn is_allowable(&self, kind: Option<TileKind>) -> bool {
        let index = match kind {
            Some(TileKind::Wall) => 0,
            Some(TileKind::Corridor(_)) => 1,
            Some(TileKind::Room { transition, .. }) => if transition { 3 } else { 2 },
            Some(TileKind::DoorWay) => 4,
            None => return false,
        };

        self.allowed[index]
    }
}

pub fn overlaps_any<T: Rect>(rect: &T, others: &[T], spacing: i32) -> bool {
    for other in others {
        if rect.overlaps(other, spacing) { return true; }
    }
    false
}

pub trait Rect {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn w(&self) -> i32;
    fn h(&self) -> i32;

    fn contains(&self, p: Point) -> bool {
        if p.x < self.x() || p.x > self.x() + self.w() { return false; }
        if p.y < self.y() || p.y > self.y() + self.h() { return false; }

        true
    }

    fn overlaps(&self, other: &Self, spacing: i32) -> bool {
        !self.not_overlaps(other, spacing)
    }

    fn not_overlaps(&self, other: &Self, spacing: i32) -> bool {
        let sp = spacing - 1;

        if self.x() > other.x() + other.w() + sp || other.x() > self.x() + self.x() + sp {
            return true;
        }

        if self.y() > other.y() + other.h() + sp || other.y() > self.y() + self.h() + sp {
            return true;
        }

        false
    }

    fn is_passable(&self, layers: &[Layer]) -> bool {
        for yi in 0..self.h() {
            for xi in 0..self.w() {
                let x = self.x() + xi;
                let y = self.y() + yi;
                if !self.point_is_passable(x, y, layers) { return false; }
            }
        }

        true
    }

    fn point_is_passable(&self, x: i32, y: i32, layers: &[Layer]) -> bool {
        for layer in layers {
            if !layer.is_passable(x, y) { return false; }
        }

        true
    }
}
