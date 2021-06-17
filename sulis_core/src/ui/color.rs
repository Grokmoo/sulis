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

use std::io::{Error, ErrorKind};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,

    #[serde(default = "float_one")]
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

fn float_one() -> f32 {
    1.0
}

fn get_component(text: &str, max: f32) -> f32 {
    let component = i32::from_str_radix(text, 16);
    match component {
        Err(_) => {
            warn!("Unable to parse color component from '{}'", text);
            1.0
        }
        Ok(c) => c as f32 / max,
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn from_string(text: &str) -> Color {
        match Color::from_str(text) {
            Ok(color) => color,
            Err(e) => {
                warn!("{}", e);
                WHITE
            }
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() == 3 || text.len() == 4 {
            let r = get_component(&text[0..1], 15.0);
            let g = get_component(&text[1..2], 15.0);
            let b = get_component(&text[2..3], 15.0);
            let a = if text.len() == 4 {
                get_component(&text[3..4], 16.0)
            } else {
                1.0
            };

            Ok(Color { r, g, b, a })
        } else if text.len() == 6 || text.len() == 8 {
            let r = get_component(&text[0..2], 255.0);
            let g = get_component(&text[2..4], 255.0);
            let b = get_component(&text[4..6], 255.0);
            let a = if text.len() == 8 {
                get_component(&text[6..8], 255.0)
            } else {
                1.0
            };

            Ok(Color { r, g, b, a })
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unable to parse color from string '{}'", text),
            ))
        }
    }
}

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const LIGHT_GRAY: Color = Color {
    r: 0.75,
    g: 0.75,
    b: 0.75,
    a: 1.0,
};
pub const GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const PURPLE: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const CYAN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
