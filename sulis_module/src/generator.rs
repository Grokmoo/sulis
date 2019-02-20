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

mod room_model;
use self::room_model::RoomModel;

mod terrain_tiles;
pub use self::terrain_tiles::{EdgesList, TerrainTiles};

mod tiles_model;
pub use self::tiles_model::{TilesModel, is_removal};

mod wall_tiles;
pub use self::wall_tiles::{WallTiles};

use std::rc::Rc;
use std::io::{Error, ErrorKind};

use sulis_core::util::Point;
use crate::area::{AreaBuilder, Layer};
use crate::{Module, TerrainKind, WallKind};

pub struct Generator {
    pub id: String,
    terrain_kind: TerrainKind,
    wall_kind: WallKind,
    grid_width: u32,
    grid_height: u32,
    room_params: RoomParams,
}

impl Generator {
    pub fn new(builder: GeneratorBuilder, module: &Module) -> Result<Generator, Error> {
        let terrain_kind = module.terrain_kind(&builder.terrain_kind).
            ok_or(Error::new(ErrorKind::InvalidInput, format!("Invalid terrain kind '{}'",
                                                                  builder.terrain_kind)))?;
        let wall_kind = module.wall_kind(&builder.wall_kind).
            ok_or(Error::new(ErrorKind::InvalidInput, format!("Invalid wall kind '{}'",
                                                              builder.wall_kind)))?;
        Ok(Generator {
            id: builder.id,
            terrain_kind,
            wall_kind,
            grid_width: builder.grid_width,
            grid_height: builder.grid_height,
            room_params: builder.rooms,
        })
    }

    pub fn gen_layer_set(&self, builder: &AreaBuilder) -> Result<Vec<Layer>, Error> {
        info!("Generating area '{}'", builder.id);
        let mut model = TilesModel::new();

        self.fill_terrain(builder, &mut model);

        let room_width = (builder.width as u32 - 2 * model.grid_width as u32) /
            (self.grid_width * model.grid_width as u32);
        let room_height = (builder.height as u32 - 2 * model.grid_height as u32) /
            (self.grid_height * model.grid_height as u32);

        let mut room_model = RoomModel::new(room_width, room_height);
        room_model.generate(&self.room_params)?;
        self.add_walls(builder, &mut model, room_model);

        info!("Generation complete.  Marshalling.");
        self.create_layers(builder, model)
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

    fn add_walls(&self, builder: &AreaBuilder, model: &mut TilesModel,
                 room_model: RoomModel) {
        info!("Generating walls from '{}'", self.wall_kind.id);
        let mut wall_index = None;
        for (index, kind) in model.wall_kinds().iter().enumerate() {
            if kind.id == self.wall_kind.id {
                wall_index = Some(index);
                break;
            }
        }

        match wall_index {
            None => {
                error!("Invalid wall kind '{}'.  This is a bug", self.wall_kind.id);
                panic!();
            },
            Some(index) => model.wall_kind(index).clone(),
        };

        for y in (0..builder.height).step_by(model.grid_height as usize) {
            for x in (0..builder.width).step_by(model.grid_width as usize) {
                model.set_wall(x as i32, y as i32, 1, wall_index);
            }
        }

        // carve out procedurally generated rooms
        let total_grid_width = model.grid_width * self.grid_width as i32;
        let total_grid_height = model.grid_height * self.grid_height as i32;

        for y in 0..room_model.height() {
            for x in 0..room_model.width() {
                if !room_model.is_wall(x, y) {
                    let offset_x = x * total_grid_width + model.grid_width;
                    let offset_y = y * total_grid_height + model.grid_height;
                    for yi in (0..total_grid_height).step_by(model.grid_height as usize) {
                        for xi in (0..total_grid_width).step_by(model.grid_width as usize) {
                            model.set_wall(xi + offset_x, yi + offset_y, 0, None);
                        }
                    }
                }
            }
        }

        // add the tiles to the model
        for y in (0..builder.height).step_by(model.grid_height as usize) {
            for x in (0..builder.width).step_by(model.grid_width as usize) {
                model.check_add_wall_border(x as i32, y as i32);
            }
        }
    }

    fn fill_terrain(&self, builder: &AreaBuilder, model: &mut TilesModel) {
        info!("Generating base terrain from '{}'", self.terrain_kind.id);
        let mut kind_index = None;
        for (index, kind) in model.terrain_kinds().iter().enumerate() {
            if kind.id == self.terrain_kind.id {
                kind_index = Some(index);
                break;
            }
        }

        let terrain = match kind_index {
            None => {
                error!("Invalid terrain kind '{}'.  This is a bug", self.terrain_kind.id);
                panic!();
            },
            Some(index) => model.terrain_kind(index).clone(),
        };

        for y in (0..builder.height).step_by(model.grid_height as usize) {
            for x in (0..builder.width).step_by(model.grid_width as usize) {
                model.add(model.gen_choice(&terrain), x as i32, y as i32);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorBuilder {
    id: String,
    terrain_kind: String,
    wall_kind: String,
    grid_width: u32,
    grid_height: u32,
    rooms: RoomParams,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoomParams {
    min_size: Point,
    max_size: Point,
    placement_attempts: u32,
    winding_chance: u32,
    connectivity: u32,
    dead_end_removal: u32,
}
