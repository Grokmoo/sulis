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
use std::rc::Rc;
use std::collections::HashMap;

use grt::util::invalid_data_error;

use module::area::{AreaBuilder, Layer};
use module::{Module, Tile, generator};

pub struct Terrain {
    pub width: i32,
    pub height: i32,
    pub layers: Vec<Layer>,
    pub entity_layer_index: usize,
    passable: Vec<bool>,
}

impl Terrain {
    pub fn new(builder: &AreaBuilder, module: &Module) -> Result<Terrain, Error> {
        let width = builder.width as i32;
        let height = builder.height as i32;
        let dim = (width * height) as usize;

        let mut layers = if builder.generate {
            let (id, tiles) = generator::generate_area(width, height, module)?;

            let layer = Layer::new(builder, id, tiles)?;
            vec![layer]
        } else {
            Terrain::validate_tiles(builder, module)?;

            let mut layer_tiles: HashMap<String, Vec<Option<Rc<Tile>>>> = HashMap::new();

            for (tile_id, locations) in &builder.terrain {
                let tile = module.tiles.get(tile_id).unwrap();

                if !layer_tiles.contains_key(&tile.layer) {
                    layer_tiles.insert(tile.layer.to_string(), vec![None;dim]);
                }

                let mut cur_layer = layer_tiles.get_mut(&tile.layer).unwrap();
                for point in locations.iter() {
                    cur_layer[point[0] + point[1] * width as usize] = Some(Rc::clone(&tile));
                }
            }

            let mut layers: Vec<Layer> = Vec::new();
            for (id, tiles) in layer_tiles {
                let layer = Layer::new(builder, id, tiles)?;
                layers.push(layer);
            }

            layers
        };

        if layers.is_empty() {
            return invalid_data_error("No tiles in area terrain");
        }

        let mut layers_sorted: Vec<Layer> = Vec::new();
        for id in builder.layers.iter() {
            let mut layer_index: Option<usize> = None;
            for (index, layer) in layers.iter().enumerate() {
                if &layer.id == id {
                    layer_index = Some(index);
                    break;
                }
            }

            let index = match layer_index {
                None => return invalid_data_error(&format!("Layer '{}' is specified in area,\
                                                  but no tiles have that layer.", id)),
                Some(index) => index,
            };

            let layer = layers.remove(index);
            layers_sorted.push(layer);
        }

        if layers.len() > 0 {
            return invalid_data_error(&format!("One or more tiles has layer '{}', but it is not\
                present in the area definition", layers[0].id));
        }

        let layers = layers_sorted;
        trace!("Created terrain for '{}' with {} layers.", builder.id, layers.len());
        let mut passable = vec![true;dim];
        for layer in layers.iter() {
            for index in 0..dim {
                if !layer.is_passable_index(index) {
                    passable[index] = false;
                }
            }
        }

        let entity_layer_index = builder.entity_layer;
        if entity_layer_index >= layers.len() {
            return invalid_data_error(
                &format!("Entity layer of {} is invalid.", entity_layer_index));
        }

        Ok(Terrain {
            width,
            height,
            layers,
            entity_layer_index,
            passable,
        })
    }

    fn validate_tiles(builder: &AreaBuilder, module: &Module) -> Result<(), Error> {
        for (tile_id, locations) in &builder.terrain {
            let tile_ref = module.tiles.get(tile_id);
            match tile_ref {
                Some(t) => t,
                None => {
                    return invalid_data_error(&format!("Tile not found '{}'", tile_id));
                }
            };

            for point in locations.iter() {
                if point.len() == 2 { continue; }

                return invalid_data_error(&format!("Point array length is not 2 in '{}'", tile_id));
            }

        }

        Ok(())
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        *self.passable.get((x + y * self.width) as usize).unwrap()
    }
}
