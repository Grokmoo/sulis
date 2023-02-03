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
use std::rc::Rc;

use sulis_core::util::invalid_data_error;

use crate::area::{AreaBuilder, Layer, PropData, Tile};
use crate::Module;

pub struct LayerSet {
    pub width: i32,
    pub height: i32,
    pub layers: Vec<Layer>,
    pub entity_layer_index: usize,
    elevation: Vec<u8>,
    pub passable: Vec<bool>,
    visible: Vec<bool>,
}

impl LayerSet {
    pub fn new(
        builder: &AreaBuilder,
        props: &[PropData],
        mut layers: Vec<Layer>,
    ) -> Result<LayerSet, Error> {
        let width = builder.width as i32;
        let height = builder.height as i32;
        let dim = (width * height) as usize;

        if layers.is_empty() {
            // layers have not been generated
            LayerSet::validate_tiles(builder)?;

            let mut layer_tiles: HashMap<String, Vec<Vec<Rc<Tile>>>> = HashMap::new();
            for layer_id in builder.layers.iter() {
                layer_tiles.insert(layer_id.to_string(), vec![Vec::new(); dim]);
            }

            for (tile_id, locations) in &builder.layer_set {
                let tile = Module::tile(tile_id).unwrap();

                if !layer_tiles.contains_key(&tile.layer) {
                    return invalid_data_error(&format!(
                        "Tile {} has undefined layer {}",
                        tile_id, tile.layer
                    ));
                }

                let cur_layer = layer_tiles.get_mut(&tile.layer).unwrap();
                for point in locations.iter() {
                    let x = point[0] as usize;
                    let y = point[1] as usize;
                    let index = x + y * width as usize;
                    if index >= dim {
                        warn!("Invalid tile location {},{}", x, y);
                        continue;
                    }
                    cur_layer[index].push(Rc::clone(&tile));
                }
            }

            for layer_id in builder.layers.iter() {
                let tiles = layer_tiles.remove(layer_id).unwrap();
                let layer = Layer::new(
                    builder.width as i32,
                    builder.height as i32,
                    layer_id.to_string(),
                    tiles,
                )?;
                layers.push(layer);
            }
        }

        if layers.is_empty() {
            return invalid_data_error("No tiles in area layer_set");
        }

        let entity_layer_index = builder.entity_layer;

        trace!(
            "Created layer_set for '{}' with {} layers.",
            builder.id,
            layers.len()
        );
        let mut passable = vec![true; dim];
        let mut visible = vec![true; dim];
        for layer in layers.iter() {
            for index in 0..dim {
                if !layer.is_passable_index(index) {
                    passable[index] = false;
                }

                if !layer.is_visible_index(index) {
                    visible[index] = false;
                }
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
                        passable[(x + y * width) as usize] = true;
                    }
                }

                for p in tile.impass.iter() {
                    let x = p.x + start_x;
                    let y = p.y + start_y;
                    passable[(x + y * width) as usize] = false;
                }
            }
        }

        for prop_data in props.iter() {
            let prop = &prop_data.prop;
            let start_x = prop_data.location.x as usize;
            let start_y = prop_data.location.y as usize;

            for p in prop.impass.iter() {
                let x = start_x + p.x as usize;
                let y = start_y + p.y as usize;
                passable[x + y * width as usize] = false;
            }

            for p in prop.invis.iter() {
                let x = start_x + p.x as usize;
                let y = start_y + p.y as usize;
                visible[x + y * width as usize] = false;
            }
        }

        if entity_layer_index >= layers.len() {
            return invalid_data_error(&format!(
                "Entity layer of {entity_layer_index} is invalid."
            ));
        }

        let elevation;
        if builder.elevation.is_empty() {
            elevation = vec![0; dim];
        } else if builder.elevation.len() != dim {
            warn!(
                "In '{}': Elevation array must be zero or dimensions length*width",
                builder.id
            );
            elevation = vec![0; dim];
        } else {
            elevation = builder.elevation.clone();
        }

        Ok(LayerSet {
            width,
            height,
            layers,
            entity_layer_index,
            elevation,
            passable,
            visible,
        })
    }

    fn validate_tiles(builder: &AreaBuilder) -> Result<(), Error> {
        for (tile_id, locations) in &builder.layer_set {
            let tile_ref = Module::tile(tile_id);
            match tile_ref {
                Some(t) => t,
                None => {
                    return invalid_data_error(&format!("Tile not found '{tile_id}'"));
                }
            };

            for point in locations.iter() {
                if point.len() == 2 {
                    continue;
                }

                return invalid_data_error(&format!(
                    "Point array length is not 2 in '{tile_id}'"
                ));
            }
        }

        Ok(())
    }

    #[inline]
    pub fn elevation(&self, x: i32, y: i32) -> u8 {
        self.elevation[(x + y * self.width) as usize]
    }

    #[inline]
    pub fn elevation_index(&self, index: usize) -> u8 {
        self.elevation[index]
    }

    #[inline]
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.passable[(x + y * self.width) as usize]
    }

    #[inline]
    pub fn is_passable_index(&self, index: usize) -> bool {
        self.passable[index]
    }

    #[inline]
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible[(x + y * self.width) as usize]
    }

    #[inline]
    pub fn is_visible_index(&self, index: usize) -> bool {
        self.visible[index]
    }
}
