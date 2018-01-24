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
use std::fmt;

use EntitySize;
use area::Terrain;

pub struct PathFinderGrid {
    pub size: Rc<EntitySize>,
    pub passable: Vec<bool>,
    pub width: i32,
    pub height: i32,
}

impl fmt::Debug for PathFinderGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PathFinderGrid of size {}\n  ", self.size.size)?;
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
    pub fn new(size: Rc<EntitySize>, terrain: &Terrain) -> PathFinderGrid {
        let width = terrain.width;
        let height = terrain.height;

        let mut passable = vec![false;(width * height) as usize];

        for y in 0..height {
            for x in 0..width {
                let mut is_passable = true;
                for p in size.points(x, y) {
                    if p.x >= width || p.y >= height {
                        is_passable = false;
                        break;
                    }
                    if !terrain.is_passable(p.x, p.y) {
                        is_passable = false;
                        break;
                    }
                }
                passable[(x + y * width) as usize] = is_passable;
            }
        }

        PathFinderGrid {
            size,
            passable,
            width,
            height,
        }
    }

    pub fn size(&self) -> i32 {
        self.size.size
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.passable[(x + y * self.width) as usize]
    }

    pub fn is_passable_index(&self, index: i32) -> bool {
        self.passable[index as usize]
    }
}
