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

use std::fmt;
use std::rc::Rc;

use crate::area::Layer;
use crate::ObjectSize;

pub struct PathFinderGrid {
    pub size: Rc<ObjectSize>,
    pub passable: Vec<bool>,
    pub width: i32,
    pub height: i32,
}

impl fmt::Debug for PathFinderGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PathFinderGrid of size {}\n  ", self.size.id)?;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.passable[(x + y * self.width) as usize] {
                    write!(f, ".")?;
                } else {
                    write!(f, "X")?;
                }
            }
            write!(f, "\n  ")?;
        }
        write!(f, "\n")
    }
}

impl PathFinderGrid {
    pub fn new(size: Rc<ObjectSize>, width: i32, height: i32, layers: &[Layer]) -> PathFinderGrid {
        let mut passable = vec![false; (width * height) as usize];

        for y in 0..height {
            for x in 0..width {
                let mut is_passable = true;
                for p in size.points(x, y) {
                    if p.x >= width || p.y >= height {
                        is_passable = false;
                        break;
                    }

                    if layers.iter().any(|layer| {
                        !layer.is_passable(p.x, p.y)
                    }) {
                        is_passable = false;
                        break;
                    }
                }
                passable[(x + y * width) as usize] = is_passable;
            }
        }

        for ref layer in layers.iter() {
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

        PathFinderGrid {
            size,
            passable,
            width,
            height,
        }
    }

    pub fn size(&self) -> &str {
        &self.size.id
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.passable[(x + y * self.width) as usize]
    }

    pub fn is_passable_index(&self, index: i32) -> bool {
        self.passable[index as usize]
    }
}
