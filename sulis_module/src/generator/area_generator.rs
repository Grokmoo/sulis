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

use std::io::Error;
use std::rc::Rc;
use std::collections::HashMap;

use sulis_core::util::{gen_rand, Point};
use crate::{Module, area::{Layer, AreaBuilder}};
use crate::generator::{WeightedList, WallKinds, RoomParams, TerrainParams, PropParams,
    EncounterParams, GeneratorBuilder, GenModel, Maze, TerrainGen, PropGen, EncounterGen,
    TileKind, TileIter, TilesModel, GeneratorOutput};

pub struct AreaGenerator {
    pub id: String,
    wall_kinds: WallKinds,
    grid_width: u32,
    grid_height: u32,
    room_params: RoomParams,
    terrain_params: TerrainParams,
    prop_params: PropParams,
    encounter_params: EncounterParams,
}

impl AreaGenerator {
    pub fn new(builder: GeneratorBuilder, module: &Module) -> Result<AreaGenerator, Error> {
        let wall_kinds_list = WeightedList::new(builder.wall_kinds, "WallKind",
                                                |id| module.wall_kind(id))?;

        Ok(AreaGenerator {
            id: builder.id,
            wall_kinds: WallKinds { kinds: wall_kinds_list },
            grid_width: builder.grid_width,
            grid_height: builder.grid_height,
            room_params: builder.rooms,
            terrain_params: TerrainParams::new(builder.terrain, module)?,
            prop_params: PropParams::new(builder.props, module)?,
            encounter_params: EncounterParams::new(builder.encounters, module)?,
        })
    }

    pub fn generate(&self, builder: &AreaBuilder) -> Result<GeneratorOutput, Error> {
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

        info!("Tile generation complete.  Creating layers.");
        let layers = self.create_layers(&model.builder, &model.model)?;

        info!("Generating props");
        let mut gen = PropGen::new(&mut model, &layers, &self.prop_params, &maze);
        let props = gen.generate()?;

        info!("Generating encounters");
        let mut gen = EncounterGen::new(&mut model, &layers, &self.encounter_params, &maze);
        let encounters = gen.generate()?;

        Ok(GeneratorOutput {
            layers,
            props,
            encounters,
        })
    }

    fn create_layers(&self, builder: &AreaBuilder, model: &TilesModel) -> Result<Vec<Layer>, Error> {
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
            Some(TileKind::Room { .. }) => {
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
        Some(TileKind::Room { .. }) => {
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
