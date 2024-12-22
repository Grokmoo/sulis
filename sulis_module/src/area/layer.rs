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

use crate::area::Tile;
use sulis_core::resource::{ResourceSet, Spritesheet};
use sulis_core::util::{invalid_data_error, Point};

pub struct Layer {
    pub id: String,
    pub width: i32,
    pub height: i32,
    display: Vec<Vec<Rc<Tile>>>,
    passable: Vec<bool>,
    visible: Vec<bool>,
    spritesheet_id: Option<String>,
    pub(crate) impass_override_tiles: Vec<(Point, Rc<Tile>)>,
}

impl Layer {
    pub fn new(
        width: i32,
        height: i32,
        id: String,
        tiles: Vec<Vec<Rc<Tile>>>,
    ) -> Result<Layer, Error> {
        let dim = (width * height) as usize;

        let mut impass_overrides = Vec::new();
        let mut display: Vec<Vec<Rc<Tile>>> = vec![Vec::new(); dim];
        let mut passable: Vec<bool> = vec![true; dim];
        let mut visible: Vec<bool> = vec![true; dim];
        let mut spritesheet_id: Option<String> = None;

        trace!("Creating layer '{}' with size: {} x {}", id, width, height);
        for (index, tile_vec) in tiles.into_iter().enumerate() {
            for tile in tile_vec {
                match spritesheet_id {
                    None => spritesheet_id = Some(tile.image_display.sheet_id.to_string()),
                    Some(ref id) => {
                        if id != &tile.image_display.sheet_id {
                            return Err(invalid_data_error(&format!(
                                "All tiles in a layer must be from the same \
                                 spritesheet: '{}' vs '{}'",
                                id, tile.id
                            )));
                        }
                    }
                }

                let base_x = (index as i32) % width;
                let base_y = (index as i32) / width;

                display[index].push(Rc::clone(&tile));

                for p in tile.impass.iter() {
                    let index = (base_x + p.x + (base_y + p.y) * width) as usize;
                    if index >= dim {
                        continue;
                    }
                    passable[index] = false;
                }

                for p in tile.invis.iter() {
                    let p_index = (base_x + p.x + (base_y + p.y) * width) as usize;
                    if p_index >= dim {
                        continue;
                    }
                    visible[p_index] = false;
                }

                if base_x + tile.width > width || base_y + tile.height > height {
                    return Err(invalid_data_error(&format!(
                        "Tile '{}' at [{}, {}] extends past area boundary.",
                        tile.id, base_x, base_y
                    )));
                }

                if tile.override_impass {
                    impass_overrides.push((Point::new(base_x, base_y), Rc::clone(&tile)));
                }
            }
        }

        Ok(Layer {
            id,
            width,
            height,
            display,
            passable,
            visible,
            spritesheet_id,
            impass_override_tiles: impass_overrides,
        })
    }

    pub fn get_spritesheet(&self) -> Option<Rc<Spritesheet>> {
        match self.spritesheet_id {
            None => None,
            Some(ref id) => ResourceSet::spritesheet(id),
        }
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible[(x + y * self.width) as usize]
    }

    pub fn is_visible_index(&self, index: usize) -> bool {
        self.visible[index]
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.passable[(x + y * self.width) as usize]
    }

    pub fn is_passable_index(&self, index: usize) -> bool {
        self.passable[index]
    }

    pub fn tiles_at(&self, x: i32, y: i32) -> &Vec<Rc<Tile>> {
        &self.display[(x + y * self.width) as usize]
    }
}
