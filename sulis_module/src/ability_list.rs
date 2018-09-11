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

use std::slice::Iter;
use std::rc::Rc;
use std::io::Error;

use sulis_core::util::{unable_to_create_error};

use {Ability, Module};

#[derive(Debug)]
pub struct Entry {
    pub ability: Rc<Ability>,
    pub position: (f32, f32),
}

#[derive(Debug)]
pub struct AbilityList {
    pub id: String,
    entries: Vec<Entry>,
}

impl AbilityList {
    pub fn new(builder: AbilityListBuilder, module: &Module) -> Result<AbilityList, Error> {
        let mut entries = Vec::new();
        for entry in builder.abilities {
            let ability = match module.abilities.get(&entry.id) {
                None => {
                    warn!("Unable to find ability '{}'", entry.id);
                    return unable_to_create_error("ability_list", &builder.id);
                }, Some(ref ability) => Rc::clone(ability),
            };

            entries.push(Entry {
                ability,
                position: entry.position,
            });
        }

        Ok(AbilityList {
            id: builder.id,
            entries,
        })
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Entry> {
        self.entries.iter()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntryBuilder {
    id: String,
    position: (f32, f32),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AbilityListBuilder {
    pub id: String,
    abilities: Vec<EntryBuilder>
}
