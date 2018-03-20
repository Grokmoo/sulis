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

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Error;

use sulis_rules::{Attribute};
use sulis_core::util::unable_to_create_error;

use {Actor, Class, Module, Race};

pub struct PrereqList {
    attributes: Option<Vec<(Attribute, u8)>>,
    levels: Vec<(Rc<Class>, u32)>,
    total_level: Option<u32>,
    race: Option<Rc<Race>>,
    abilities: Vec<String>, // we can't store the actual ability here because it might
                            // not be parsed yet
}

impl PrereqList {
    pub fn new(builder: PrereqListBuilder, module: &Module) -> Result<PrereqList, Error> {
        let mut levels = Vec::new();
        if let Some(builder_levels) = builder.levels {
            for (class_id, level) in builder_levels {
                let class = match module.classes.get(&class_id) {
                    None => {
                        warn!("No match for class '{}'", class_id);
                        return unable_to_create_error("prereq_list", "prereqs class");
                    }, Some(class) => Rc::clone(class)
                };

                levels.push((class, level));
            }
        }

        let race = if let Some(ref race_id) = builder.race {
            match module.races.get(race_id) {
                None => {
                    warn!("No match for race '{}'", race_id);
                    return unable_to_create_error("prereq_list", "prereqs race");
                }, Some(race) => Some(Rc::clone(race))
            }
        } else {
            None
        };

        Ok(PrereqList {
            attributes: builder.attributes,
            levels,
            total_level: builder.total_level,
            race,
            abilities: builder.abilities.unwrap_or(Vec::new()),
        })
    }

    pub fn meets(&self, actor: &Rc<Actor>) -> bool {
        if let Some(ref attrs) = self.attributes {
            for &(attr, amount) in attrs.iter() {
                if actor.attributes.get(attr) < amount { return false; }
            }
        }

        for &(ref class, level) in self.levels.iter() {
            if actor.levels(class) < level { return false; }
        }

        if let Some(total_level) = self.total_level {
            if actor.total_level < total_level { return false; }
        }

        if let Some(ref race) = self.race {
            if &actor.race != race { return false; }
        }

        for ref ability_id in self.abilities.iter() {
            if !actor.has_ability_with_id(ability_id) { return false; }
        }

        true
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrereqListBuilder {
    pub attributes: Option<Vec<(Attribute, u8)>>,
    pub levels: Option<HashMap<String, u32>>,
    pub total_level: Option<u32>,
    pub race: Option<String>,
    pub abilities: Option<Vec<String>>,
}

