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

#[derive(Default, Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct Border {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Border {
    pub fn uniform(border: i32) -> Border {
        Border {
            top: border,
            bottom: border,
            left: border,
            right: border,
        }
    }

    pub fn vertical(&self) -> i32 {
        self.top + self.bottom
    }

    pub fn horizontal(&self) -> i32 {
        self.right + self.left
    }
}
