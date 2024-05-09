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

use serde::Deserialize;

use crate::rules::Attribute;

use crate::{Actor, Module};

#[derive(Debug, Clone)]
pub struct PrereqList {
    pub attributes: Option<Vec<(Attribute, u8)>>,

    /// A class and associated level in that class that is required
    /// Unlike other prereqs, this is an OR condition, meaning
    /// if the vec is non-empty, at least one of the entries must
    /// be met
    pub levels: Vec<(String, u32)>,

    pub total_level: Option<u32>,
    pub race: Option<String>, // we can't store the actual ability or race because they
    pub abilities: Vec<String>, // might not be read in yet
}

impl PrereqList {
    /// Merges two PrereqList options.  See `merge`
    pub fn merge_optional(a: &Option<PrereqList>, b: &Option<PrereqList>) -> Option<PrereqList> {
        match a {
            None => b.clone(),
            Some(a) => match b {
                None => Some(a.clone()),
                Some(b) => Some(PrereqList::merge(a, b)),
            },
        }
    }

    /// Create a PrereqList that is the union of the two argument PrereqLists.  The
    /// new list will be one that requires all prereqs from both arguments to be met.
    /// Since two different race prereqs cannot be met simultaneously, emits a warning
    /// and picks the race prereq from `a` in the event that `a` and `b` both have
    /// different race prereqs.
    pub fn merge(a: &PrereqList, b: &PrereqList) -> PrereqList {
        // take the max of all individual attr requirements
        let mut attributes: HashMap<_, _> = a.attributes.as_ref().map_or(HashMap::new(), |attrs| {
            attrs
                .iter()
                .map(|(attr, amount)| (*attr, *amount))
                .collect()
        });

        for (attr, amount) in b.attributes.as_ref().unwrap_or(&Vec::new()) {
            let cur = attributes.get(attr).unwrap_or(&0);
            let amount = std::cmp::max(*cur, *amount);
            attributes.insert(*attr, amount);
        }

        // take the max of all individual level requirements
        let mut levels: HashMap<_, _> = a
            .levels
            .iter()
            .map(|(id, level)| (id.to_string(), *level))
            .collect();
        for (id, level) in b.levels.iter() {
            let cur = *levels.get(id).unwrap_or(&0);
            let level = std::cmp::max(cur, *level);
            levels.insert(id.to_string(), level);
        }

        // take the max of total level requirements
        let total_level = std::cmp::max(a.total_level.unwrap_or(0), b.total_level.unwrap_or(0));
        let total_level = if total_level == 0 {
            None
        } else {
            Some(total_level)
        };

        let race = match &a.race {
            None => b.race.clone(),
            Some(race) => {
                if let Some(race_b) = &b.race {
                    if race != race_b {
                        warn!(
                            "Merging prereqs with incompatible race requirements: {} and {}",
                            race, race_b
                        );
                    }
                }
                Some(race.to_string())
            }
        };

        // get all unique abilities, use hashset to dedup
        let mut abilities = a.abilities.clone();
        abilities.extend(b.abilities.iter().cloned());
        let abilities: HashSet<_> = abilities.into_iter().collect();

        PrereqList {
            attributes: Some(attributes.into_iter().collect()),
            levels: levels.into_iter().collect(),
            total_level,
            race,
            abilities: abilities.into_iter().collect(),
        }
    }

    pub fn new(builder: PrereqListBuilder) -> Result<PrereqList, Error> {
        let mut levels = Vec::new();
        if let Some(builder_levels) = builder.levels {
            for (class_id, level) in builder_levels {
                levels.push((class_id, level));
            }
        }

        Ok(PrereqList {
            attributes: builder.attributes,
            levels,
            total_level: builder.total_level,
            race: builder.race,
            abilities: builder.abilities.unwrap_or_default(),
        })
    }

    pub fn meets(&self, actor: &Rc<Actor>) -> bool {
        if let Some(ref attrs) = self.attributes {
            for &(attr, amount) in attrs.iter() {
                if actor.attributes.get(attr) < amount {
                    return false;
                }
            }
        }

        let mut class_level_met = self.levels.is_empty();
        for &(ref class_id, level) in self.levels.iter() {
            let class = match Module::class(class_id) {
                None => {
                    warn!("Invalid class '{}' found when matching prereqs", class_id);
                    continue;
                }
                Some(class) => class,
            };
            if actor.levels(&class) >= level {
                class_level_met = true;
                break;
            }
        }

        if !class_level_met {
            return false;
        }

        if let Some(total_level) = self.total_level {
            if actor.total_level < total_level {
                return false;
            }
        }

        if let Some(ref race) = self.race {
            if &actor.race.name != race {
                return false;
            }
        }

        for ability_id in self.abilities.iter() {
            if !actor.has_ability_with_id(ability_id) {
                return false;
            }
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
