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

mod terrain_tiles;
pub use self::terrain_tiles::{EdgesList, TerrainTiles};

mod tiles_model;
pub use self::tiles_model::{TilesModel, is_removal};

mod wall_tiles;
pub use self::wall_tiles::{WallTiles};

use std::rc::Rc;
use std::io::{Error, ErrorKind};

use crate::area::{AreaBuilder, Layer};
use crate::{Module, TerrainKind};

pub struct Generator {
    pub id: String,
    terrain_kind: TerrainKind,
}

impl Generator {
    pub fn new(builder: GeneratorBuilder, module: &Module) -> Result<Generator, Error> {
        let terrain_kind = module.terrain_kind(&builder.terrain_kind).
            ok_or(Error::new(ErrorKind::InvalidInput, format!("Invalid terrain kind '{}'",
                                                                  builder.terrain_kind)))?;
        Ok(Generator {
            id: builder.id,
            terrain_kind,
        })
    }

    pub fn gen_layer_set(&self, builder: &AreaBuilder) -> Result<Vec<Layer>, Error> {
        debug!("Generating layer set for '{}'", builder.id);
        let mut out = Vec::new();

        let mut model = TilesModel::new();

        let mut kind_index = None;
        for (index, kind) in model.terrain_kinds().iter().enumerate() {
            if kind.id == self.terrain_kind.id {
                kind_index = Some(index);
                break;
            }
        }

        let terrain = match kind_index {
            None => {
                error!("Invalid terrain kind '{}'", self.terrain_kind.id);
                panic!();
            },
            Some(index) => model.terrain_kind(index).clone(),
        };

        for y in (0..builder.height).step_by(model.grid_height as usize) {
            for x in (0..builder.width).step_by(model.grid_width as usize) {
                model.add(model.gen_choice(&terrain), x as i32, y as i32);
            }
        }

        for (id, tiles_data) in model.iter() {
            let mut tiles = vec![Vec::new(); (builder.width * builder.height) as usize];
            for (p, tile) in tiles_data.iter() {
                let index = (p.x + p.y * builder.width as i32) as usize;
                tiles[index].push(Rc::clone(tile));
            }

            out.push(Layer::new(builder, id.to_string(), tiles)?);
        }

        Ok(out)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorBuilder {
    id: String,
    terrain_kind: String,
}
