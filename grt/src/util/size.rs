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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::ops;
use std::cmp;

use ui::Border;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn as_zero() -> Size {
        Size { width: 0, height: 0 }
    }

    pub fn new(width: i32, height: i32) -> Size {
        Size {
            width,
            height,
        }
    }

    pub fn from(other: &Size) -> Size {
        Size {
            width: other.width,
            height: other.height,
        }
    }

    pub fn inner(&self, border: &Border) -> Size {
        Size {
            width: self.width - border.left - border.right,
            height: self.height - border.top - border.bottom,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn product(&self) -> i32 {
        self.width * self.height

    }

    pub fn add(&self, w: i32, h: i32) -> Size {
        Size::new(self.width + w, self.height + h)
    }

    pub fn add_mut(&mut self, width: i32, height: i32) {
        self.width += width;
        self.height += height;
    }

    pub fn add_width(&mut self, width: i32) {
        self.width += width;
    }

    pub fn add_height(&mut self, height: i32) {
        self.height += height;
    }

    pub fn add_width_from(&mut self, size: &Size) {
        self.width += size.width;
    }

    pub fn add_height_from(&mut self, size: &Size) {
        self.height += size.height;
    }

    pub fn set(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_width(&mut self, width: i32) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: i32) {
        self.height = height;
    }

    pub fn max_from(&mut self, other: &Size) {
        self.width = cmp::max(self.width, other.width);
        self.height = cmp::max(self.height, other.height);
    }

    pub fn min_from(&mut self, other: &Size) {
        self.width = cmp::min(self.width, other.width);
        self.height = cmp::min(self.height, other.height);
    }

    pub fn max(&mut self, width: i32, height: i32) {
        self.width = cmp::max(self.width, width);
        self.height = cmp::max(self.height, height);
    }

    pub fn min(&mut self, width: i32, height: i32) {
        self.width = cmp::min(self.width, width);
        self.height = cmp::min(self.height, height);
    }
}

impl ops::Add<Size> for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Size {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl ops::Sub<Size> for Size {
    type Output = Size;

    fn sub(self, rhs: Size) -> Size {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
