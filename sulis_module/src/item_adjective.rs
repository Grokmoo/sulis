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

use sulis_core::resource::ResourceBuilder;
use sulis_core::serde_json;
use sulis_core::serde_yaml;

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

impl ResourceBuilder for ItemAdjective {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn from_json(data: &str) -> Result<ItemAdjective, Error> {
        let resource: ItemAdjective = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ItemAdjective, Error> {
        let resource: Result<ItemAdjective, serde_yaml::Error>
            = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData,
                                         format!("{}", error)))
        }
    }
}
