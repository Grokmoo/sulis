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
use std::collections::HashMap;
use std::io::Error;

use sulis_core::resource::ResourceBuilder;
use sulis_core::util::{invalid_data_error, unable_to_create_error};
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_rules::{AttributeList, BonusList};

use {Ability, Module};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Kit {
    pub name: String,
    pub description: String,
    pub default_attributes: AttributeList,
    pub starting_equipment: Vec<String>,
}

pub struct Class {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bonuses_per_level: BonusList,
    ability_choices: HashMap<u32, Vec<Vec<Rc<Ability>>>>,
    pub kits: Vec<Kit>,
}

impl PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        self.id == other.id
    }
}

impl Class {
    pub fn new(builder: ClassBuilder, module: &Module) -> Result<Class, Error> {
        if builder.bonuses_per_level.attack.is_some() {
            warn!("Must not specify an attack for a class.");
            return unable_to_create_error("class", &builder.id);
        }

        let mut ability_choices = HashMap::new();
        for (level, list_vec) in builder.ability_choices {
            let mut vec = Vec::new();

            for list in list_vec {
                let mut abilities = Vec::new();

                for ability_id in list {
                    let ability = match module.abilities.get(&ability_id) {
                        None => {
                            warn!("Unable to find ability '{}'", ability_id);
                            return unable_to_create_error("class", &builder.id);
                        }, Some(ref ability) => Rc::clone(ability),
                    };

                    abilities.push(ability);
                }

                vec.push(abilities);
            }


            ability_choices.insert(level, vec);
        }

        Ok(Class {
            id: builder.id,
            name: builder.name,
            description: builder.description,
            bonuses_per_level: builder.bonuses_per_level,
            kits: builder.kits,
            ability_choices,
        })
    }

    pub fn ability_choices(&self, level: u32) -> Vec<Vec<Rc<Ability>>> {
        match self.ability_choices.get(&level) {
            None => Vec::new(),
            Some(choices) => choices.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClassBuilder {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bonuses_per_level: BonusList,
    pub ability_choices: HashMap<u32, Vec<Vec<String>>>,
    pub kits: Vec<Kit>,
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
