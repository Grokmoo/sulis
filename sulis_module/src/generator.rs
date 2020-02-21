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
pub(crate) use self::encounter_gen::{EncounterGen, EncounterParams, EncounterParamsBuilder};

mod feature_gen;
use self::feature_gen::{FeatureGen, FeatureParams, FeatureParamsBuilder};

mod maze;
use self::maze::{Maze, TileKind};

mod prop_gen;
pub(crate) use self::prop_gen::{PropGen, PropParams, PropParamsBuilder};

mod terrain_gen;
use self::terrain_gen::{TerrainGen, TerrainParams, TerrainParamsBuilder};

mod terrain_tiles;
pub use self::terrain_tiles::{EdgesList, TerrainTiles};

mod tiles_model;
pub use self::tiles_model::{is_removal, TilesModel};

mod transition_gen;
use self::transition_gen::{
    TransitionGen, TransitionOutput, TransitionParams, TransitionParamsBuilder,
};

mod wall_tiles;
pub use self::wall_tiles::WallTiles;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use crate::area::{EncounterDataBuilder, Layer, LocationChecker, PathFinderGrid, PropDataBuilder};
use crate::{ObjectSize, WallKind};
use sulis_core::util::{Point, ReproducibleRandom};

pub struct LayerListLocationChecker {
    grid: PathFinderGrid,
}
impl LayerListLocationChecker {
    pub fn new(
        width: i32,
        height: i32,
        layers: &[Layer],
        size: Rc<ObjectSize>,
    ) -> LayerListLocationChecker {
        // set up passability matrix based on the layers
        let mut pass = vec![false; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                pass[(x + y * width) as usize] = layers.iter().all(|l| l.is_passable(x, y));
            }
        }

        for layer in layers.iter() {
            for &(point, ref tile) in layer.impass_override_tiles.iter() {
                let start_x = point.x;
                let start_y = point.y;
                let end_x = start_x + tile.width;
                let end_y = start_y + tile.height;

                for y in start_y..end_y {
                    for x in start_x..end_x {
                        pass[(x + y * width) as usize] = true;
                    }
                }

                for p in tile.impass.iter() {
                    let x = p.x + start_x;
                    let y = p.y + start_y;
                    pass[(x + y * width) as usize] = false;
                }
            }
        }

        let grid = PathFinderGrid::new(size, width, height, &pass);
        LayerListLocationChecker { grid }
    }
}

impl LocationChecker for LayerListLocationChecker {
    fn passable(&self, x: i32, y: i32) -> Option<bool> {
        Some(self.grid.is_passable(x, y))
    }
}

pub struct WeightedList<T> {
    total_weight: u32,
    entries: Vec<(String, T, u32)>,
}

impl<T> WeightedList<T> {
    pub fn new<F>(
        kinds: HashMap<String, WeightedEntry>,
        name: &str,
        getter: F,
    ) -> Result<WeightedList<T>, Error>
    where
        F: Fn(&str) -> Option<T>,
    {
        let mut entries = Vec::new();
        let mut total_weight = 0;
        for (id, entry) in kinds {
            let t = getter(&id).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid {} '{}'", name, id),
                )
            })?;

            total_weight += entry.weight;
            entries.push((id, t, entry.weight));
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        if total_weight == 0 || entries.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Must specify at least one {}", name),
            ));
        }

        Ok(WeightedList {
            entries,
            total_weight,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn pick(&self, rand: &mut ReproducibleRandom) -> &T {
        if self.entries.len() == 1 || self.total_weight == 1 {
            return &self.entries[0].1;
        }

        let roll = rand.gen(0, self.total_weight);

        let mut cur_weight = 0;
        for (_, kind, weight) in self.entries.iter() {
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
    fn pick_index(&self, rand: &mut ReproducibleRandom, model: &TilesModel) -> Option<usize> {
        let wall_kind = self.kinds.pick(rand);
        let mut wall_index = None;
        for (index, kind) in model.wall_kinds().iter().enumerate() {
            if kind.id == wall_kind.id {
                wall_index = Some(index);
                break;
            }
        }

        if wall_index.is_none() {
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
}

pub(crate) struct GenModel {
    model: TilesModel,
    area_width: i32,
    area_height: i32,
    total_grid_size: Point,
    region_overfill_edges: HashMap<usize, usize>,
    rand: ReproducibleRandom,
}

impl GenModel {
    fn new(
        width: i32,
        height: i32,
        rand: ReproducibleRandom,
        gen_grid_width: i32,
        gen_grid_height: i32,
    ) -> GenModel {
        let model = TilesModel::new();
        let gen_grid_size = Point::new(gen_grid_width, gen_grid_height);
        let total_grid_size = Point::new(
            gen_grid_size.x * model.grid_width,
            gen_grid_size.y * model.grid_height,
        );
        GenModel {
            model,
            area_width: width,
            area_height: height,
            total_grid_size,
            region_overfill_edges: HashMap::new(),
            rand,
        }
    }

    fn region_size(&self) -> (u32, u32) {
        let x = (self.area_width - 2 * self.model.grid_width) / self.total_grid_size.x;
        let y = (self.area_height - 2 * self.model.grid_height) / self.total_grid_size.y;
        (x as u32, y as u32)
    }

    fn to_region_coords(&self, x: i32, y: i32) -> (i32, i32) {
        (
            (x - self.model.grid_width) / self.total_grid_size.x,
            (y - self.model.grid_height) / self.total_grid_size.y,
        )
    }

    fn from_region_coords(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x * self.total_grid_size.x + self.model.grid_width,
            y * self.total_grid_size.y + self.model.grid_height,
        )
    }

    fn tiles(&self) -> TileIter {
        TileIter::new(self)
    }

    pub fn rand(&self) -> &ReproducibleRandom {
        &self.rand
    }

    pub fn rand_mut(&mut self) -> &mut ReproducibleRandom {
        &mut self.rand
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

    fn new(model: &GenModel) -> TileIter {
        TileIter {
            x: 0,
            y: 0,
            max_x: model.area_width,
            max_y: model.area_height,
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
        if self.y >= self.max_y {
            return None;
        }

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
    min_passable_size: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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

        RegionKinds { allowed }
    }

    pub(crate) fn check_coords(&self, maze: &Maze, p1: Point, p2: Point) -> bool {
        for y in p1.y..=p2.y {
            for x in p1.x..=p2.x {
                let t = maze.tile_checked(x, y);
                if !self.is_allowable(t) {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_allowable(&self, kind: Option<TileKind>) -> bool {
        let index = match kind {
            Some(TileKind::Wall) => 0,
            Some(TileKind::Corridor(_)) => 1,
            Some(TileKind::Room { transition, .. }) => {
                if transition {
                    3
                } else {
                    2
                }
            }
            Some(TileKind::DoorWay) => 4,
            None => return false,
        };

        self.allowed[index]
    }
}

pub fn overlaps_any<T: Rect>(rect: &T, others: &[T], spacing: i32) -> bool {
    for other in others {
        if rect.overlaps(other, spacing) {
            return true;
        }
    }
    false
}

pub trait Rect {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn w(&self) -> i32;
    fn h(&self) -> i32;

    fn contains(&self, p: Point) -> bool {
        if p.x < self.x() || p.x > self.x() + self.w() {
            return false;
        }
        if p.y < self.y() || p.y > self.y() + self.h() {
            return false;
        }

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
                if !self.point_is_passable(x, y, layers) {
                    return false;
                }
            }
        }

        true
    }

    fn point_is_passable(&self, x: i32, y: i32, layers: &[Layer]) -> bool {
        for layer in layers {
            if !layer.is_passable(x, y) {
                return false;
            }
        }

        true
    }
}
