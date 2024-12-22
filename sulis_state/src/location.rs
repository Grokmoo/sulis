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

use std::fmt::{Debug, Formatter, Result};

use crate::AreaState;
use sulis_core::util::Point;
use sulis_module::Area;

#[derive(Clone, Eq)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub area_id: String,

    pub area_width: i32,
    pub area_height: i32,
}

impl Debug for Location {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "{{ {},{} in {} }}", self.x, self.y, self.area_id)
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.x == other.x && self.y == other.y && self.area_id == other.area_id
    }
}

impl Location {
    pub fn new(x: i32, y: i32, area: &Area) -> Location {
        Location {
            x,
            y,
            area_id: area.id.clone(),
            area_width: area.width,
            area_height: area.height,
        }
    }

    pub fn from_point(p: Point, area: &Area) -> Location {
        Location {
            x: p.x,
            y: p.y,
            area_id: area.id.clone(),
            area_width: area.width,
            area_height: area.height,
        }
    }

    pub fn to_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn coords_valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.area_width && y < self.area_height
    }


    pub fn is_in(&self, area_state: &AreaState) -> bool {
        self.area_id == area_state.area.area.id
    }

    pub fn is_in_area_id(&self, id: &str) -> bool {
        self.area_id == id
    }
}
