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

use rand::{self, Rng};
use std::io::{Error};
use std::rc::Rc;

use sulis_core::util::unable_to_create_error;
use {Actor, Module};

struct Entry {
    actor: Rc<Actor>,
    weight: u32,
    always: bool,
}

pub struct Encounter {
    pub id: String,
    pub auto_spawn: bool,
    min_gen_actors: u32,
    max_gen_actors: u32,
    entries: Vec<Entry>,
    total_weight: u32,
}

impl Encounter {
    pub fn new(builder: EncounterBuilder, module: &Module) -> Result<Encounter, Error> {
        if builder.entries.len() == 0 {
            warn!("Cannot have an encounter with no entries");
            return unable_to_create_error("encounter", &builder.id);
        }

        let mut entries = Vec::new();
        let mut total_weight = 0;
        for entry in builder.entries {
            let actor = match module.actors.get(&entry.id) {
                None => {
                    warn!("no actor '{}' found", entry.id);
                    return unable_to_create_error("encounter", &builder.id);
                }, Some(actor) => Rc::clone(actor),
            };

            total_weight += entry.weight;
            entries.push(Entry {
                actor,
                weight: entry.weight,
                always: entry.always,
            });
        }

        Ok(Encounter {
            id: builder.id,
            auto_spawn: builder.auto_spawn,
            min_gen_actors: builder.min_gen_actors,
            max_gen_actors: builder.max_gen_actors,
            entries,
            total_weight,
        })
    }

    fn gen_actor(&self) -> Option<Rc<Actor>> {
        let roll = rand::thread_rng().gen_range(0, self.total_weight);
        let mut cur_weight = 0;
        for entry in self.entries.iter() {
            cur_weight += entry.weight;
            if roll < cur_weight {
                return Some(Rc::clone(&entry.actor));
            }
        }

        None
    }

    pub fn gen_actors(&self) -> Vec<Rc<Actor>> {
        let mut actors = Vec::new();

        let num = rand::thread_rng().gen_range(self.min_gen_actors, self.max_gen_actors + 1);
        for _ in 0..num {
            match self.gen_actor() {
                None => {
                    warn!("Unable to generate actor for encounter '{}'", self.id);
                    continue;
                }, Some(actor) => {
                    actors.push(actor);
                }
            }
        }

        for entry in self.entries.iter() {
            if entry.always { actors.push(Rc::clone(&entry.actor)); }
        }

        actors
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EncounterBuilder {
    pub id: String,
    pub auto_spawn: bool,
    min_gen_actors: u32,
    max_gen_actors: u32,
    entries: Vec<EntryBuilder>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntryBuilder {
    id: String,

    #[serde(default)]
    weight: u32,

    #[serde(default)]
    always: bool,
}
