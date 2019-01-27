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
use std::ops;
use std::cmp;

use crate::ui::Border;

#[derive(Copy, Clone, Default, Deserialize, Serialize, Eq, Hash, PartialEq, Ord)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<cmp::Ordering> {
        Some(self.cmp_hash().cmp(&other.cmp_hash()))
    }
}

impl Point {
    fn cmp_hash(&self) -> i32 {
        self.x + self.y * 65536
    }

    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fn from(other: &Point) -> Point {
        Point { x: other.x, y: other.y }
    }

    pub fn from_tuple_i32(vals: (i32, i32)) -> Point {
        Point { x: vals.0, y: vals.1 }
    }

    pub fn from_tuple(other: (u32, u32)) -> Point {
        Point { x: other.0 as i32, y: other.1 as i32 }
    }

    pub fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn mult_mut(&mut self, val: i32) {
        self.x *= val;
        self.y *= val;
    }

    pub fn add(&self, x: i32, y: i32) -> Point {
        Point { x: &self.x + x, y: &self.y + y }
    }

    pub fn add_mut(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }

    pub fn add_x(&mut self, x: i32) {
        self.x += x;
    }

    pub fn add_y(&mut self, y: i32) {
        self.y += y;
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn min(&mut self, x: i32, y: i32) {
        self.x = cmp::min(self.x, x);
        self.y = cmp::min(self.y, y);
    }

    pub fn max(&mut self, x: i32, y: i32) {
        self.x = cmp::max(self.x, x);
        self.y = cmp::max(self.y, y);
    }

    pub fn new_vec(data: Vec<(i32, i32)>) -> Vec<Point> {
        let mut vec = Vec::with_capacity(data.len());

        for (x, y) in data {
            vec.push(Point::new(x, y));
        }

        vec
    }

    pub fn inner(&self, border: &Border) -> Point {
        Point { x: self.x + border.left, y: self.y + border.top }
    }

    /// Returns true if this point's x coordinate is in
    /// the interval [0, max_x) and the y coordinate is in
    /// the interval [0, max_y).  Returns false otherwise.
    pub fn in_bounds(&self, max_x: i32, max_y: i32) -> bool {
        if self.x < 0 || self.y < 0 { return false; }
        if self.x >= max_x || self.y >= max_y { return false; }
        true
    }

    /// Returns the eucliden distance between these two points
    pub fn dist(&self, other: &Point) -> f32 {
        (((self.x - other.x) * (self.x - other.x) +
            (self.y - other.y) * (self.y - other.y)) as f32).sqrt()
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{},{}}}", self.x, self.y)
    }
}
