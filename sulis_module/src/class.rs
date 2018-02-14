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

use std::io::Error;

use sulis_core::resource::ResourceBuilder;
use sulis_core::util::invalid_data_error;
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_rules::BonusList;

pub struct Class {
    pub id: String,
    pub name: String,
    pub bonuses_per_level: BonusList,
}

impl PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        self.id == other.id
    }
}

impl Class {
    pub fn new(builder: ClassBuilder) -> Result<Class, Error> {
        if builder.bonuses_per_level.base_damage.is_some() {
            warn!("Must not specify a base damage for a class.");
            return invalid_data_error(&format!("Unable to create class '{}'", builder.id));
        }

        Ok(Class {
            id: builder.id,
            name: builder.name,
            bonuses_per_level: builder.bonuses_per_level,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClassBuilder {
    pub id: String,
    pub name: String,
    pub bonuses_per_level: BonusList,
}

impl ResourceBuilder for ClassBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ClassBuilder, Error> {
        let resource: ClassBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ClassBuilder, Error> {
        let resource: Result<ClassBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
