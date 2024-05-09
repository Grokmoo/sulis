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
use std::rc::Rc;

use serde::{Serialize, Deserialize};

use crate::rules::{AttributeList, BonusList};
use sulis_core::util::{unable_to_create_error, ExtInt};

use crate::{Ability, AbilityList, InventoryBuilder, Module};

#[derive(Debug)]
pub struct Kit {
    pub name: String,
    pub description: String,
    pub default_attributes: AttributeList,
    pub starting_inventory: InventoryBuilder,
    pub starting_abilities: Vec<Rc<Ability>>,
}

#[derive(Debug, Clone)]
pub struct Upgrades {
    pub ability_choices: Vec<Rc<AbilityList>>,
    pub group_uses_per_encounter: Vec<(String, ExtInt)>,
    pub group_uses_per_day: Vec<(String, ExtInt)>,
    pub stats: HashMap<String, ExtInt>,
}

#[derive(Debug)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bonuses_per_level: BonusList,
    upgrades: HashMap<u32, Upgrades>,
    max_level_upgrades: Upgrades,
    starting_abilities: Vec<Rc<Ability>>,
    pub kits: Vec<Kit>,
    pub stats: Vec<ClassStat>,
}

impl PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        self.id == other.id
    }
}

impl Class {
    pub fn new(builder: ClassBuilder, module: &Module) -> Result<Class, Error> {
        let mut upgrades = HashMap::new();
        let mut max_level = 0;
        for (level, upgrades_builder) in builder.upgrades {
            if level == 0 {
                warn!("Invalid upgrade level 0");
                return unable_to_create_error("class", &builder.id);
            }
            max_level = level.max(max_level);

            let mut ability_choices = Vec::new();
            for ability_list_id in upgrades_builder.ability_choices {
                let ability_list = match module.ability_lists.get(&ability_list_id) {
                    None => {
                        warn!("Unable to find ability list '{}'", ability_list_id);
                        return unable_to_create_error("class", &builder.id);
                    }
                    Some(ability_list) => Rc::clone(ability_list),
                };

                ability_choices.push(ability_list);
            }

            let group_uses_per_day = upgrades_builder.group_uses_per_day;
            let group_uses_per_encounter = upgrades_builder.group_uses_per_encounter;
            let stats = upgrades_builder.stats;

            for id in stats.keys() {
                if !builder.stats.iter().any(|stat| id == &stat.id) {
                    warn!("Unable to find stat for class upgrades: '{}'", id);
                    return unable_to_create_error("class", &builder.id);
                }
            }

            let upgrades_for_level = Upgrades {
                ability_choices,
                group_uses_per_encounter,
                group_uses_per_day,
                stats,
            };

            upgrades.insert(level, upgrades_for_level);
        }

        if upgrades.is_empty() {
            warn!("Each class must specify upgrades for at least 1 level.");
            return unable_to_create_error("class", &builder.id);
        }

        let max_level_upgrades = upgrades.get(&max_level).unwrap().clone();

        if builder.kits.is_empty() {
            warn!("Each class must specify at least one kit.");
            return unable_to_create_error("class", &builder.id);
        }

        let mut abilities = Vec::new();
        for ability_id in builder.starting_abilities {
            let ability = match module.abilities.get(&ability_id) {
                None => {
                    warn!("Unable to find ability '{}'", ability_id);
                    return unable_to_create_error("class", &builder.id);
                }
                Some(ability) => Rc::clone(ability),
            };

            abilities.push(ability);
        }

        let mut kits = Vec::new();
        for kit_builder in builder.kits {
            let mut abilities = Vec::new();
            for ability_id in kit_builder.starting_abilities {
                let ability = match module.abilities.get(&ability_id) {
                    None => {
                        warn!("Unable to find ability '{}'", ability_id);
                        return unable_to_create_error("class", &builder.id);
                    }
                    Some(ability) => Rc::clone(ability),
                };

                abilities.push(ability);
            }

            kits.push(Kit {
                name: kit_builder.name,
                description: kit_builder.description,
                default_attributes: kit_builder.default_attributes,
                starting_inventory: kit_builder.starting_inventory,
                starting_abilities: abilities,
            });
        }

        Ok(Class {
            id: builder.id,
            name: builder.name,
            description: builder.description,
            bonuses_per_level: builder.bonuses_per_level,
            kits,
            upgrades,
            max_level_upgrades,
            starting_abilities: abilities,
            stats: builder.stats,
        })
    }

    pub fn displayed_class_stat(&self) -> Option<&ClassStat> {
        self.stats.iter().find(|stat| stat.display)
    }

    pub fn starting_abilities(&self) -> impl Iterator<Item = &Rc<Ability>> {
        self.starting_abilities.iter()
    }

    pub fn ability_choices(&self, level: u32) -> &[Rc<AbilityList>] {
        match self.upgrades.get(&level) {
            None => &self.max_level_upgrades.ability_choices,
            Some(upgrades) => &upgrades.ability_choices,
        }
    }

    pub fn stats_max(&self, level: u32) -> &HashMap<String, ExtInt> {
        match self.upgrades.get(&level) {
            None => &self.max_level_upgrades.stats,
            Some(upgrades) => &upgrades.stats,
        }
    }

    pub fn group_uses_per_day(&self, level: u32) -> &[(String, ExtInt)] {
        match self.upgrades.get(&level) {
            None => &self.max_level_upgrades.group_uses_per_day,
            Some(upgrades) => &upgrades.group_uses_per_day,
        }
    }

    pub fn group_uses_per_encounter(&self, level: u32) -> &[(String, ExtInt)] {
        match self.upgrades.get(&level) {
            None => &self.max_level_upgrades.group_uses_per_encounter,
            Some(upgrades) => &upgrades.group_uses_per_encounter,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClassStat {
    pub id: String,
    pub name: String,
    pub display: bool,
    pub reset_per_encounter: bool,
    pub reset_per_day: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct KitBuilder {
    pub name: String,
    pub description: String,
    pub default_attributes: AttributeList,
    pub starting_inventory: InventoryBuilder,
    pub starting_abilities: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpgradesBuilder {
    ability_choices: Vec<String>,

    #[serde(default)]
    group_uses_per_encounter: Vec<(String, ExtInt)>,

    #[serde(default)]
    group_uses_per_day: Vec<(String, ExtInt)>,

    #[serde(default)]
    stats: HashMap<String, ExtInt>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClassBuilder {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bonuses_per_level: BonusList,
    pub upgrades: HashMap<u32, UpgradesBuilder>,
    pub starting_abilities: Vec<String>,
    pub kits: Vec<KitBuilder>,

    #[serde(default)]
    pub stats: Vec<ClassStat>,
}
