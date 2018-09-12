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

/// An adjective is a modifier that affects the stats of
/// an item in a given way.  Items can have zero, one, or
/// many adjectives.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemAdjective {
    pub id: String,
    pub name: String,
}

impl PartialEq for ItemAdjective {
    fn eq(&self, other: &ItemAdjective) -> bool {
        self.id == other.id
    }
}
