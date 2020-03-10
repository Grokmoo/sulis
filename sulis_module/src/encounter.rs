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

use std::collections::{HashMap, HashSet};
use std::io::Error;
use std::rc::Rc;

use crate::{Actor, Module};
use sulis_core::io::SoundSource;
use sulis_core::resource::ResourceSet;
use sulis_core::util::{gen_rand, unable_to_create_error};

struct Entry {
    actor: Rc<Actor>,
    unique_id: Option<String>,
    weight: u32,
    always: bool,
    limit: Option<u32>,
}

pub struct Encounter {
    pub id: String,
    pub music: Option<SoundSource>,
    pub auto_spawn: bool,
    min_gen_actors: u32,
    max_gen_actors: u32,
    entries: Vec<Entry>,
    total_weight: u32,
}

impl Encounter {
    pub fn new(builder: EncounterBuilder, module: &Module) -> Result<Encounter, Error> {
        if builder.entries.is_empty() {
            warn!("Cannot have an encounter with no entries");
            return unable_to_create_error("encounter", &builder.id);
        }

        let mut ids = HashSet::new();

        let mut entries = Vec::new();
        let mut total_weight = 0;
        for entry in builder.entries {
            if !entry.always {
                if ids.contains(&entry.id) {
                    warn!("Duplicate entry '{}'", entry.id);
                    return unable_to_create_error("encounter", &builder.id);
                }

                ids.insert(entry.id.to_string());
            }

            let actor = match module.actors.get(&entry.id) {
                None => {
                    warn!("no actor '{}' found", entry.id);
                    return unable_to_create_error("encounter", &builder.id);
                }
                Some(actor) => Rc::clone(actor),
            };

            if entry.always && entry.limit.is_some() {
                warn!("Cannot set a limit on an always generated entry.");
                return unable_to_create_error("encounter", &builder.id);
            }

            if entry.always && entry.weight > 0 {
                warn!("Cannot set a weight on an always generated entry.");
                return unable_to_create_error("encounter", &builder.id);
            }

            total_weight += entry.weight;
            entries.push(Entry {
                actor,
                unique_id: entry.unique_id,
                weight: entry.weight,
                limit: entry.limit,
                always: entry.always,
            });
        }

        let music = match &builder.music {
            None => None,
            Some(id) => Some(ResourceSet::sound(id)?),
        };

        Ok(Encounter {
            id: builder.id,
            music,
            auto_spawn: builder.auto_spawn,
            min_gen_actors: builder.min_gen_actors,
            max_gen_actors: builder.max_gen_actors,
            entries,
            total_weight,
        })
    }

    fn gen_actor(&self, count: &mut HashMap<String, u32>) -> Option<(Rc<Actor>, Option<String>)> {
        if self.total_weight == 0 {
            return None;
        }

        // try to gen a maximum of 100 times
        for _ in 0..100 {
            let index = match self.gen_roll() {
                None => continue,
                Some(index) => index,
            };

            let entry = &self.entries[index];

            if let Some(limit) = entry.limit {
                let cur_count = *count.get(&entry.actor.id).unwrap_or(&0);
                if cur_count >= limit {
                    continue;
                }
                count.insert(entry.actor.id.to_string(), cur_count + 1);
            }

            return Some((Rc::clone(&entry.actor), entry.unique_id.clone()));
        }

        warn!("Unable to generate a valid actor after max attempts");
        None
    }

    fn gen_roll(&self) -> Option<usize> {
        let roll = gen_rand(0, self.total_weight);
        let mut cur_weight = 0;
        for (index, entry) in self.entries.iter().enumerate() {
            cur_weight += entry.weight;
            if roll < cur_weight {
                return Some(index);
            }
        }

        None
    }

    pub fn gen_actors(&self) -> Vec<(Rc<Actor>, Option<String>)> {
        let mut actors = Vec::new();

        let total_num = gen_rand(self.min_gen_actors, self.max_gen_actors + 1);

        let mut count = HashMap::new();
        let mut cur_num = 0;

        while cur_num < total_num {
            let actor = match self.gen_actor(&mut count) {
                None => {
                    warn!("Unable to generate actor for encounter '{}'", self.id);
                    return actors;
                }
                Some(actor) => actor,
            };

            actors.push(actor);

            cur_num += 1;
        }

        for entry in self.entries.iter() {
            if entry.always {
                actors.push((Rc::clone(&entry.actor), entry.unique_id.clone()));
            }
        }

        actors
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EncounterBuilder {
    pub id: String,
    pub music: Option<String>,
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

    limit: Option<u32>,

    #[serde(default)]
    unique_id: Option<String>,
}
