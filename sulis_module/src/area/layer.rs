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

use std::rc::Rc;
use std::io::Error;

use sulis_core::resource::{ResourceSet, Spritesheet};
use sulis_core::util::invalid_data_error;
use area::{AreaBuilder, Tile};

pub struct Layer {
    pub id: String,
    pub width: i32,
    pub height: i32,
    display: Vec<Option<Rc<Tile>>>,
    passable: Vec<bool>,
    visible: Vec<bool>,
    spritesheet_id: String,
}

impl Layer {
    pub fn new(builder: &AreaBuilder, id: String,
               tiles: Vec<Option<Rc<Tile>>>) -> Result<Layer, Error> {
        let width = builder.width as i32;
        let height = builder.height as i32;
        let dim = (width * height) as usize;

        let mut display: Vec<Option<Rc<Tile>>> = vec![None;dim];
        let mut passable: Vec<bool> = vec![true;dim];
        let mut visible: Vec<bool> = vec![true;dim];
        let mut spritesheet_id: Option<String> = None;

        trace!("Creating layer '{}' with size: {} x {}", id, width, height);
        for (index, tile) in tiles.into_iter().enumerate() {
            let tile = match tile {
                None => continue,
                Some(tile) => tile,
            };

            match spritesheet_id {
                None => spritesheet_id = Some(tile.image_display.id.to_string()),
                Some(ref id) => {
                    if id != &tile.image_display.id {
                        return invalid_data_error(&format!("All tiles in a layer must be from the same \
                                                           spritesheet: '{}' vs '{}'", id, tile.id));
                    }
                }
            }

            display[index] = Some(Rc::clone(&tile));

            let base_x = (index as i32) % width;
            let base_y = (index as i32) / width;

            for p in tile.impass.iter() {
                passable[(base_x + p.x + (base_y + p.y) * width) as usize] = false;
            }

            for p in tile.invis.iter() {
                visible[(base_x + p.x + (base_y + p.y) * width) as usize] = false;
            }

            if base_x + tile.width > width || base_y + tile.height > height {
                return invalid_data_error(
                    &format!("Tile '{}' at [{}, {}] extends past area boundary.",
                             tile.id, base_x, base_y));
            }
        }

        let spritesheet_id = match spritesheet_id {
            None => return invalid_data_error("Empty layer"),
            Some(id) => id,
        };

        Ok(Layer {
            id,
            width,
            height,
            display,
            passable,
            visible,
            spritesheet_id,
        })
    }

    pub fn get_spritesheet(&self) -> Rc<Spritesheet> {
        ResourceSet::get_spritesheet(&self.spritesheet_id).unwrap()
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

    pub fn tile_at(&self, x: i32, y: i32) -> &Option<Rc<Tile>> {
        &self.display[(x + y * self.width) as usize]
    }
}
