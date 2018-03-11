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
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::resource::{ResourceBuilder};
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_core::util::unable_to_create_error;

use {Actor, Module};

struct Entry {
    actor: Rc<Actor>,
    weight: u32,
}

pub struct Encounter {
    pub id: String,
    min_actors: u32,
    max_actors: u32,
    entries: Vec<Entry>,
}

impl Encounter {
    pub fn new(builder: EncounterBuilder, module: &Module) -> Result<Encounter, Error> {
        let mut entries = Vec::new();
        for (actor_id, entry) in builder.entries {
            let actor = match module.actors.get(&actor_id) {
                None => {
                    warn!("no actor '{}' found", actor_id);
                    return unable_to_create_error("encounter", &builder.id);
                }, Some(actor) => Rc::clone(actor),
            };

            entries.push(Entry {
                actor,
                weight: entry.weight,
            });
        }

        Ok(Encounter {
            id: builder.id,
            min_actors: builder.min_actors,
            max_actors: builder.max_actors,
            entries,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EncounterBuilder {
    pub id: String,
    min_actors: u32,
    max_actors: u32,
    entries: HashMap<String, EntryBuilder>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntryBuilder {
    weight: u32,
}

impl ResourceBuilder for EncounterBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<EncounterBuilder, Error> {
        let resource: EncounterBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<EncounterBuilder, Error> {
        let resource: Result<EncounterBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
