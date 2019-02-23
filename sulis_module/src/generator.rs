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

mod maze;
use self::maze::{Maze, TileKind};

mod terrain_gen;
use self::terrain_gen::{TerrainGen, TerrainParams, TerrainParamsBuilder};

mod terrain_tiles;
pub use self::terrain_tiles::{EdgesList, TerrainTiles};

mod tiles_model;
pub use self::tiles_model::{TilesModel, is_removal};

mod wall_tiles;
pub use self::wall_tiles::{WallTiles};

use std::collections::{HashMap};
use std::rc::Rc;
use std::io::{Error, ErrorKind};

use sulis_core::util::{Point, gen_rand};
use crate::area::{AreaBuilder, Layer};
use crate::{Module, WallKind};

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

pub struct Generator {
    pub id: String,
    wall_kinds: WallKinds,
    grid_width: u32,
    grid_height: u32,
    room_params: RoomParams,
    terrain_params: TerrainParams,
}

impl Generator {
    pub fn new(builder: GeneratorBuilder, module: &Module) -> Result<Generator, Error> {
        let wall_kinds_list = WeightedList::new(builder.wall_kinds, "WallKind",
                                                |id| module.wall_kind(id))?;

        Ok(Generator {
            id: builder.id,
            wall_kinds: WallKinds { kinds: wall_kinds_list },
            grid_width: builder.grid_width,
            grid_height: builder.grid_height,
            room_params: builder.rooms,
            terrain_params: TerrainParams::new(builder.terrain, module)?,
        })
    }

    pub fn gen_layer_set(&self, builder: &AreaBuilder) -> Result<Vec<Layer>, Error> {
        info!("Generating area '{}'", builder.id);
        let mut model = GenModel::new(builder, self.grid_width as i32, self.grid_height as i32);

        let (room_width, room_height) = model.region_size();
        let mut maze = Maze::new(room_width, room_height);

        let open_locs: Vec<Point> = builder.transitions.iter().map(|t| {
            let (x, y) = model.to_region_coords(t.from.x, t.from.y);
            Point::new(x, y)
        }).collect();
        maze.generate(&self.room_params, &open_locs)?;
        self.add_walls(&mut model, &maze);

        info!("Generating terrain");
        let mut gen = TerrainGen::new(&mut model, &self.terrain_params, &maze);
        gen.generate();

        // add the tiles to the model
        for p in model.tiles() {
            model.model.check_add_wall_border(p.x, p.y);
            model.model.check_add_terrain(p.x, p.y);
            model.model.check_add_terrain_border(p.x, p.y);
        }

        info!("Generation complete.  Marshalling.");
        self.create_layers(&model.builder, model.model)
    }

    fn create_layers(&self, builder: &AreaBuilder, model: TilesModel) -> Result<Vec<Layer>, Error> {
        let mut out = Vec::new();
        for (id, tiles_data) in model.iter() {
            let mut tiles = vec![Vec::new(); (builder.width * builder.height) as usize];
            for (p, tile) in tiles_data.iter() {
                if p.x >= builder.width as i32|| p.y >= builder.height as i32 { continue; }
                let index = (p.x + p.y * builder.width as i32) as usize;
                tiles[index].push(Rc::clone(tile));
            }

            out.push(Layer::new(builder, id.to_string(), tiles)?);
        }

        Ok(out)
    }

    fn pick_wall_kind(&self,
                      model: &GenModel,
                      invert: bool,
                      region: usize,
                      mapped: &mut HashMap<usize, Option<usize>>) -> (u8, Option<usize>) {
        if !invert { return (0, None); }

        if mapped.contains_key(&region) {
            (1, mapped[&region])
        } else {
            let wall_index = self.wall_kinds.pick_index(&model.model);
            mapped.insert(region, wall_index);
            (1, wall_index)
        }
    }

    fn add_walls(&self, model: &mut GenModel, maze: &Maze) {
        info!("Generating walls");

        // either carve rooms out or put walls in
        if self.room_params.invert {
            for p in model.tiles() {
                model.model.set_wall(p.x, p.y, 0, None);
            }
        } else {
            let wall_index = self.wall_kinds.pick_index(&model.model);
            for p in model.tiles() {
                model.model.set_wall(p.x, p.y, 1, wall_index);
            }
        }

        let mut mapped = HashMap::new();

        // carve out procedurally generated rooms
        let room_iter = TileIter::simple(maze.width(), maze.height());
        for p_room in room_iter {
            let region = match maze.region(p_room.x, p_room.y) {
                None => continue,
                Some(region) => region,
            };

            let neighbors = maze.neighbors(p_room.x, p_room.y);
            let (elev, wall_kind) = self.pick_wall_kind(model, self.room_params.invert,
                                                        region, &mut mapped);

            let (offset_x, offset_y) = model.from_region_coords(p_room.x, p_room.y);
            let (tot_gw, tot_gh) = (model.total_grid_size.x, model.total_grid_size.y);
            let (gw, gh) = (model.model.grid_width, model.model.grid_height);
            self.carve_wall(model, neighbors, Point::new(offset_x, offset_y),
                Point::new(gw as i32, gh as i32), Point::new(tot_gw, tot_gh), elev, wall_kind);
        }
    }

    fn carve_wall(&self, model: &mut GenModel, neighbors: [Option<TileKind>; 5],
                  offset: Point, step: Point, max: Point,
                  elev: u8, wall_kind: Option<usize>) {
        let edge_choice = match neighbors[0] {
            Some(TileKind::Corridor(region)) => {
                if gen_rand(1, 101) < self.room_params.corridor_edge_overfill_chance {
                    // pregen a single potential overfill for each corridor, preventing
                    // both sides from becoming blocked at room coord intersection
                    Some(*model.region_overfill_edges.entry(region).or_insert(gen_rand(1, 5)))
                } else {
                    None
                }
            },
            Some(TileKind::Room(_)) => {
                if gen_rand(1, 101) < self.room_params.room_edge_overfill_chance {
                    Some(gen_rand(1, 5))
                } else {
                    None
                }
            },
            _ => None,
        };

        let model = &mut model.model;
        // carve borders with random roughness
        for x in (step.x..max.x - step.x).step_by(step.x as usize) {
            if !is_rough_edge(&neighbors, 1, edge_choice) {
                model.set_wall(x + offset.x, offset.y, elev, wall_kind);
            }
        }

        for y in (step.y..max.y - step.y).step_by(step.y as usize) {
            if !is_rough_edge(&neighbors, 2, edge_choice) {
                model.set_wall(offset.x + max.x - step.x, y + offset.y, elev, wall_kind);
            }
        }

        for x in (step.x..max.x - step.x).step_by(step.x as usize) {
            if !is_rough_edge(&neighbors, 3, edge_choice) {
                model.set_wall(x + offset.x, offset.y + max.y - step.y, elev, wall_kind);
            }
        }

        for y in (step.y..max.y - step.y).step_by(step.y as usize) {
            if !is_rough_edge(&neighbors, 4, edge_choice) {
                model.set_wall(offset.x, y + offset.y, elev, wall_kind);
            }
        }

        // carve corners
        if !is_rough_edge(&neighbors, 1, edge_choice) &&
            !is_rough_edge(&neighbors, 2, edge_choice) {
            model.set_wall(offset.x + max.x - step.x, offset.y, elev, wall_kind);
        }

        if !is_rough_edge(&neighbors, 2, edge_choice) &&
            !is_rough_edge(&neighbors, 3, edge_choice) {
            model.set_wall(offset.x + max.x - step.x, offset.y + max.y - step.y, elev, wall_kind);
        }

        if !is_rough_edge(&neighbors, 3, edge_choice) &&
            !is_rough_edge(&neighbors, 4, edge_choice) {
            model.set_wall(offset.x, offset.y + max.y - step.y, elev, wall_kind);
        }

        if !is_rough_edge(&neighbors, 4, edge_choice) &&
            !is_rough_edge(&neighbors, 1, edge_choice) {
            model.set_wall(offset.x, offset.y, elev, wall_kind);
        }

        // carve center
        for y in (step.y..max.y - step.y).step_by(step.y as usize) {
            for x in (step.x..max.x - step.x).step_by(step.x as usize) {
                model.set_wall(x + offset.x, y + offset.y, elev, wall_kind);
            }
        }
    }
}

fn is_rough_edge(neighbors: &[Option<TileKind>; 5], index: usize,
                 edge_choice: Option<usize>) -> bool {
    if neighbors[index] != Some(TileKind::Wall) { return false; }

    match neighbors[0] {
        Some(TileKind::Room(_)) => {
            match edge_choice {
                None => false,
                Some(_) => true,
            }
        },
        Some(TileKind::Corridor(_)) => {
            match edge_choice {
                None => false,
                Some(choice) => choice == index
            }
        },
        _ => false,
    }
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
