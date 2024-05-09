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
use std::io::Error;

use serde::Deserialize;

use sulis_core::util::{gen_rand, unable_to_create_error};

use crate::{ItemState, Module};

const MAX_DEPTH: u32 = 10;

#[derive(Debug)]
struct Entry {
    id: String,
    weight: u32,
    quantity: [u32; 2],

    adjective1_total_weight: u32,
    adjective1: Vec<(String, u32)>,

    adjective2_total_weight: u32,
    adjective2: Vec<(String, u32)>,

    variant_total_weight: u32,
    variant: Vec<(usize, u32)>,
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

    weighted_entries: Vec<Entry>,
    total_entries_weight: u32,

    probability_entries: Vec<Entry>,

    sub_lists: Vec<Entry>,
}

impl LootList {
    pub fn new(builder: LootListBuilder, module: &Module) -> Result<LootList, Error> {
        let mut total_generate_weight = 0;
        let mut generate = Vec::new();
        for (num_items, weight) in builder.generate {
            total_generate_weight += weight;
            generate.push(Generate { num_items, weight });
        }

        let mut total_entries_weight = 0;
        let mut weighted_entries = Vec::new();
        for (id, entry) in builder.weighted_entries {
            let entry = LootList::create_entry(&builder.id, module, id, entry)?;
            total_entries_weight += entry.weight;

            weighted_entries.push(entry);
        }

        let mut probability_entries = Vec::new();
        for (id, entry) in builder.probability_entries {
            let entry = LootList::create_entry(&builder.id, module, id, entry)?;
            probability_entries.push(entry);
        }

        let mut sub_lists = Vec::new();
        for (id, entry) in builder.sub_lists {
            let entry = LootList::create_sublist_entry(&builder.id, id, entry)?;
            sub_lists.push(entry);
        }

        Ok(LootList {
            id: builder.id,
            generate,
            total_generate_weight,
            weighted_entries,
            total_entries_weight,
            probability_entries,
            sub_lists,
        })
    }

    fn create_sublist_entry(
        builder_id: &str,
        id: String,
        entry_in: EntryBuilder,
    ) -> Result<Entry, Error> {
        if !entry_in.adjective1.is_empty()
            || !entry_in.adjective2.is_empty()
            || !entry_in.variant.is_empty()
        {
            warn!(
                "Item adjective and variant may not be specified in loot sub_list entries: '{}'",
                id
            );
            return unable_to_create_error("loot_list", builder_id);
        }

        let (min_qty, max_qty) = match entry_in.quantity {
            None => (1, 1),
            Some(qty) => (qty[0], qty[1]),
        };

        Ok(Entry {
            id,
            weight: entry_in.weight,
            quantity: [min_qty, max_qty],
            adjective1: Vec::new(),
            adjective1_total_weight: 0,
            adjective2: Vec::new(),
            adjective2_total_weight: 0,
            variant: Vec::new(),
            variant_total_weight: 0,
        })
    }

    fn create_entry(
        builder_id: &str,
        module: &Module,
        id: String,
        entry_in: EntryBuilder,
    ) -> Result<Entry, Error> {
        if !module.items.contains_key(&id) {
            warn!("Unable to find item '{}'", id);
            return unable_to_create_error("loot_list", builder_id);
        }

        let (min_qty, max_qty) = match entry_in.quantity {
            None => (1, 1),
            Some(qty) => (qty[0], qty[1]),
        };

        let mut adjective1 = Vec::new();
        let mut adjective1_total_weight = 0;
        for (id, weight) in entry_in.adjective1 {
            adjective1_total_weight += weight;
            if id == "none" {
                continue;
            }
            if !module.item_adjectives.contains_key(&id) {
                warn!("Unable to find item adjective '{}'", id);
                return unable_to_create_error("loot_list", builder_id);
            }
            adjective1.push((id, weight));
        }

        let mut adjective2 = Vec::new();
        let mut adjective2_total_weight = 0;
        for (id, weight) in entry_in.adjective2 {
            adjective2_total_weight += weight;
            if id == "none" {
                continue;
            }
            if !module.item_adjectives.contains_key(&id) {
                warn!("Unable to find item adjective '{}'", id);
                return unable_to_create_error("loot_list", builder_id);
            }
            adjective2.push((id, weight));
        }

        let mut variant = Vec::new();
        let mut variant_total_weight = 0;
        for (id, weight) in entry_in.variant {
            variant_total_weight += weight;
            if id == "none" {
                continue;
            }
            let value = match id.parse() {
                Ok(val) => val,
                Err(_) => {
                    warn!("Variant IDs must be parsable integers: '{}'", id);
                    return unable_to_create_error("loot_list", builder_id);
                }
            };
            variant.push((value, weight));
        }

        Ok(Entry {
            id,
            weight: entry_in.weight,
            quantity: [min_qty, max_qty],
            adjective1,
            adjective1_total_weight,
            adjective2,
            adjective2_total_weight,
            variant,
            variant_total_weight,
        })
    }

    pub fn generate_with_chance(&self, chance: u32) -> Vec<(u32, ItemState)> {
        let roll = gen_rand(1, 101);
        if chance >= roll {
            self.generate_internal(0)
        } else {
            Vec::new()
        }
    }

    pub fn generate(&self) -> Vec<(u32, ItemState)> {
        self.generate_internal(0)
    }

    fn generate_internal(&self, depth: u32) -> Vec<(u32, ItemState)> {
        if depth >= MAX_DEPTH {
            warn!(
                "Exceeded maximum sub list depth of {}.  \
                 This is most likely caused by a circular reference.",
                MAX_DEPTH
            );
            return Vec::new();
        }

        let num_items = self.gen_num_items();

        let mut items = Vec::new();
        if num_items > 0 {
            for _ in 0..num_items {
                if let Some(item) = self.gen_item() {
                    items.push(item);
                }
            }
        }

        for entry in self.probability_entries.iter() {
            let roll = gen_rand(0, 100);
            if roll < entry.weight {
                let quantity = if entry.quantity[0] == entry.quantity[1] {
                    entry.quantity[0]
                } else {
                    gen_rand(entry.quantity[0], entry.quantity[1] + 1)
                };

                let adjectives = self.gen_adjectives(entry);
                let item = match Module::create_get_item(&entry.id, &adjectives) {
                    None => {
                        warn!(
                            "Unable to create item '{}' with '{:?}'",
                            entry.id, adjectives
                        );
                        continue;
                    }
                    Some(item) => item,
                };
                let variant = self.gen_variant(entry);
                items.push((quantity, ItemState::new(item, variant)));
            }
        }

        for entry in self.sub_lists.iter() {
            let sub_list = match Module::loot_list(&entry.id) {
                None => {
                    warn!(
                        "No loot list with ID '{}' found for sublist of '{}'",
                        entry.id, self.id
                    );
                    continue;
                }
                Some(list) => list,
            };

            let roll = gen_rand(0, 100);
            if roll < entry.weight {
                let times = if entry.quantity[0] == entry.quantity[1] {
                    entry.quantity[0]
                } else {
                    gen_rand(entry.quantity[0], entry.quantity[1] + 1)
                };

                for _ in 0..times {
                    let subitems = sub_list.generate_internal(depth + 1);
                    for (quantity, item) in subitems {
                        items.push((quantity, item));
                    }
                }
            }
        }

        items
    }

    fn gen_adjectives(&self, entry: &Entry) -> Vec<String> {
        let mut result = Vec::new();
        if entry.adjective1_total_weight > 0 {
            let roll = gen_rand(0, entry.adjective1_total_weight);

            let mut cur_weight = 0;
            for (id, weight) in entry.adjective1.iter() {
                cur_weight += weight;
                if roll < cur_weight {
                    result.push(id.clone());
                    break;
                }
            }
        }

        if entry.adjective2_total_weight > 0 {
            let roll = gen_rand(0, entry.adjective2_total_weight);

            let mut cur_weight = 0;
            for (id, weight) in entry.adjective2.iter() {
                cur_weight += weight;
                if roll < cur_weight {
                    result.push(id.clone());
                    break;
                }
            }
        }

        result
    }

    fn gen_variant(&self, entry: &Entry) -> Option<usize> {
        if entry.variant_total_weight > 0 {
            let roll = gen_rand(0, entry.variant_total_weight);
            let mut cur_weight = 0;
            for (id, weight) in entry.variant.iter() {
                cur_weight += weight;
                if roll < cur_weight {
                    return Some(*id);
                }
            }
        }

        None
    }

    fn gen_item(&self) -> Option<(u32, ItemState)> {
        let roll = gen_rand(0, self.total_entries_weight);

        let mut cur_weight = 0;
        for entry in self.weighted_entries.iter() {
            cur_weight += entry.weight;
            if roll < cur_weight {
                let quantity = if entry.quantity[0] == entry.quantity[1] {
                    entry.quantity[0]
                } else {
                    gen_rand(entry.quantity[0], entry.quantity[1] + 1)
                };

                let adjectives = self.gen_adjectives(entry);
                let item = match Module::create_get_item(&entry.id, &adjectives) {
                    None => {
                        warn!(
                            "Unable to create item '{}' with '{:?}'",
                            entry.id, adjectives
                        );
                        continue;
                    }
                    Some(item) => item,
                };
                let variant = self.gen_variant(entry);
                return Some((quantity, ItemState::new(item, variant)));
            }
        }

        None
    }

    fn gen_num_items(&self) -> u32 {
        if self.total_generate_weight == 0 {
            return 0;
        }

        let roll = gen_rand(0, self.total_generate_weight);

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
    quantity: Option<[u32; 2]>,
    #[serde(default)]
    adjective1: HashMap<String, u32>,
    #[serde(default)]
    adjective2: HashMap<String, u32>,

    #[serde(default)]
    variant: HashMap<String, u32>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LootListBuilder {
    pub id: String,
    #[serde(default)]
    generate: HashMap<u32, u32>,
    #[serde(default)]
    weighted_entries: HashMap<String, EntryBuilder>,
    #[serde(default)]
    probability_entries: HashMap<String, EntryBuilder>,

    #[serde(default)]
    sub_lists: HashMap<String, EntryBuilder>,
}
