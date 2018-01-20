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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Attribute {
    Strength,
    Dexterity,
    Endurance,
    Perception,
    Intellect,
    Wisdom,
}

use self::Attribute::*;

const ATTRS_LIST: [Attribute; 6] = [ Strength, Dexterity, Endurance, Perception, Intellect, Wisdom ];

pub const BASE_VALUE: u8 = 10;

impl Attribute {
    pub fn from(text: &str) -> Option<Attribute> {
        Some(match text {
            "Strength" => Strength,
            "Dexterity" => Dexterity,
            "Endurance" => Endurance,
            "Perception" => Perception,
            "Intellect" => Intellect,
            "Wisdom" => Wisdom,
            _ => return None,
        })
    }

    pub fn name(&self) -> &str {
        match *self {
            Strength => "Strength",
            Dexterity => "Dexterity",
            Endurance => "Endurance",
            Perception => "Perception",
            Intellect => "Intellect",
            Wisdom => "Wisdom",
        }
    }

    pub fn short_name(&self) -> &str {
        match *self {
            Strength => "str",
            Dexterity => "dex",
            Endurance => "end",
            Perception => "per",
            Intellect => "int",
            Wisdom => "wis",
        }
    }

    pub fn iter() -> AttributeIterator {
        AttributeIterator { index: 0 }
    }
}

pub struct AttributeIterator {
    index: usize,
}

impl Iterator for AttributeIterator {
    type Item = Attribute;
    fn next(&mut self) -> Option<Attribute> {
        if self.index == 6 {
            None
        } else {
            let next = ATTRS_LIST[self.index];
            self.index += 1;
            Some(next)
        }
    }
}
