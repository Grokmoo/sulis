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
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use sulis_core::resource::{ResourceBuilder};
use sulis_core::serde_yaml;
use sulis_core::util::unable_to_create_error;

use {Item, Module};

#[derive(Debug)]
struct Entry {
    item: Rc<Item>,
    weight: u32,
}

#[derive(Debug)]
struct Generate {
    num_items: u32,
    weight: u32,
}

#[derive(Debug)]
pub struct LootList {
    pub id: String,
    generate: Vec<Generate>,
    total_generate_weight: u32,

    entries: Vec<Entry>,
    total_entries_weight: u32,
}

impl LootList {
    pub fn new(builder: LootListBuilder, module: &Module) -> Result<LootList, Error> {
        let mut total_generate_weight = 0;
        let mut generate = Vec::new();
        for (num_items, weight) in builder.generate {
            total_generate_weight += weight;
            generate.push(Generate {
                num_items,
                weight,
            });
        }

        let mut total_entries_weight = 0;
        let mut entries = Vec::new();
        for (id, entry) in builder.entries {
            total_entries_weight += entry.weight;
            let item = match module.items.get(&id) {
                None => {
                    warn!("Unable to find item '{}'", id);
                    return unable_to_create_error("loot_list", &builder.id);
                }, Some(item) => Rc::clone(item),
            };

            entries.push(Entry {
                item,
                weight: entry.weight,
            });
        }

        Ok(LootList {
            id: builder.id,
            generate,
            total_generate_weight,
            entries,
            total_entries_weight,
        })
    }

    pub fn generate_with_chance(&self, chance: u32) -> Vec<Rc<Item>> {
        let roll = rand::thread_rng().gen_range(1, 101);
        if chance >= roll {
            self.generate()
        } else {
            Vec::new()
        }
    }

    pub fn generate(&self) -> Vec<Rc<Item>> {
        let num_items = self.gen_num_items();

        let mut items = Vec::new();
        for _ in 0..num_items {
            if let Some(item) = self.gen_item() {
                items.push(item);
            }
        }

        items
    }

    fn gen_item(&self) -> Option<Rc<Item>> {
        let roll = rand::thread_rng().gen_range(0, self.total_entries_weight);

        let mut cur_weight = 0;
        for entry in self.entries.iter() {
            cur_weight += entry.weight;
            if roll < cur_weight {
                return Some(Rc::clone(&entry.item));
            }
        }

        None
    }

    fn gen_num_items(&self) -> u32 {
        let roll = rand::thread_rng().gen_range(0, self.total_generate_weight);

        let mut cur_gen_weight = 0;
        for generate in self.generate.iter() {
            cur_gen_weight += generate.weight;
            if roll < cur_gen_weight {
                return generate.num_items;
            }
        }

        0
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct EntryBuilder {
    weight: u32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LootListBuilder {
    pub id: String,
    generate: HashMap<u32, u32>,
    entries: HashMap<String, EntryBuilder>,
}

impl ResourceBuilder for LootListBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_yaml(data: &str) -> Result<LootListBuilder, Error> {
        let resource: Result<LootListBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
