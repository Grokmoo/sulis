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

use serde::{Deserialize, Serialize};

use sulis_core::util::ExtInt;
use sulis_module::{Ability, Actor, Class, Faction, Module, StatList};

/// Persistent Stats, that are not computed from the base StatList, are
/// saved, and may persist between actions
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PStats {
    hp: i32,
    ap: u32,
    overflow_ap: i32,
    xp: u32,
    has_level_up: bool,

    #[serde(default)]
    inventory_locked: bool,

    #[serde(skip)] // will be computed on load anyway
    threatened_by: Vec<usize>,

    #[serde(skip)]
    threatening: Vec<usize>,

    pub(crate) current_group_uses_per_encounter: HashMap<String, ExtInt>,
    pub(crate) current_group_uses_per_day: HashMap<String, ExtInt>,

    #[serde(default)]
    pub(crate) current_class_stats: HashMap<String, ExtInt>,
    pub(crate) faction: Faction,

    #[serde(default)]
    disabled: bool,

    #[serde(skip)]
    base_class: Option<Rc<Class>>,
}

impl PStats {
    pub fn new(actor: &Actor) -> PStats {
        PStats {
            hp: 0,
            ap: 0,
            overflow_ap: 0,
            xp: actor.xp,
            has_level_up: false,
            inventory_locked: false,
            threatened_by: Vec::new(),
            threatening: Vec::new(),
            current_group_uses_per_encounter: HashMap::new(),
            current_group_uses_per_day: HashMap::new(),
            current_class_stats: HashMap::new(),
            faction: actor.faction(),
            disabled: false,
            base_class: Some(actor.base_class()),
        }
    }

    pub fn load(&mut self, base_class: Rc<Class>) {
        self.base_class = Some(base_class);
    }

    pub fn remove_class_stats(&mut self, ability: &Ability) {
        let active = match &ability.active {
            None => return,
            Some(active) => active,
        };

        let stats = match active
            .class_stats
            .get(&self.base_class.as_ref().unwrap().id)
        {
            None => return,
            Some(stats) => stats,
        };
        for (stat, amount) in stats {
            if *amount == 0 {
                continue;
            }

            let cur = *self
                .current_class_stats
                .get(stat)
                .unwrap_or(&ExtInt::Int(0));
            self.current_class_stats
                .insert(stat.to_string(), cur - *amount);
        }
    }

    pub fn has_required_class_stats(&self, ability: &Ability) -> bool {
        let active = match &ability.active {
            None => return true,
            Some(active) => active,
        };

        let stats = match active
            .class_stats
            .get(&self.base_class.as_ref().unwrap().id)
        {
            None => return true,
            Some(stats) => stats,
        };
        for (stat, amount) in stats {
            if *amount == 0 {
                continue;
            }
            let cur = match self.current_class_stats.get(stat) {
                None => return false,
                Some(cur) => cur,
            };

            if cur.less_than(*amount) {
                return false;
            }
        }

        true
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns true if the parent entity is threatened by the entity
    /// with the specified index, false otherwise
    pub fn is_threatened_by(&self, index: usize) -> bool {
        self.threatened_by.contains(&index)
    }

    pub fn is_threatened(&self) -> bool {
        !self.threatened_by.is_empty()
    }

    pub fn add_threatening(&mut self, index: usize) {
        if !self.threatening.contains(&index) {
            self.threatening.push(index);
        }
    }

    pub fn remove_threatening(&mut self, index: usize) {
        self.threatening.retain(|x| *x != index);
    }

    pub fn add_threatener(&mut self, index: usize) {
        if !self.threatened_by.contains(&index) {
            self.threatened_by.push(index);
        }
    }

    pub fn remove_threatener(&mut self, index: usize) {
        self.threatened_by.retain(|x| *x != index);
    }

    pub fn is_inventory_locked(&self) -> bool {
        self.inventory_locked
    }

    pub fn set_inventory_locked(&mut self, locked: bool) {
        self.inventory_locked = locked;
    }

    pub fn hp(&self) -> i32 {
        self.hp
    }

    pub fn ap(&self) -> u32 {
        self.ap
    }

    pub fn overflow_ap(&self) -> i32 {
        self.overflow_ap
    }

    pub fn has_level_up(&self) -> bool {
        self.has_level_up
    }

    pub fn xp(&self) -> u32 {
        self.xp
    }

    pub fn set_overflow_ap(&mut self, ap: i32) {
        let rules = Module::rules();
        self.overflow_ap = ap;

        if self.overflow_ap > rules.max_overflow_ap {
            self.overflow_ap = rules.max_overflow_ap;
        } else if self.overflow_ap < rules.min_overflow_ap {
            self.overflow_ap = rules.min_overflow_ap;
        }
    }

    pub fn add_ap(&mut self, ap: u32) {
        self.ap += ap;
    }

    pub fn remove_ap(&mut self, ap: u32) {
        if ap > self.ap {
            self.ap = 0;
        } else {
            self.ap -= ap;
        }
    }

    pub fn remove_hp(&mut self, hp: u32) {
        let hp = hp as i32;
        if hp > self.hp {
            self.hp = 0;
        } else {
            self.hp -= hp;
        }
    }

    pub fn add_class_stat(&mut self, stat: &str, amount: u32, max: ExtInt) {
        let amount = match self.current_class_stats.get(stat) {
            None => ExtInt::Int(amount),
            Some(cur) => {
                if cur.is_infinite() {
                    return;
                }
                *cur + amount
            }
        };
        let amount = ExtInt::min(amount, max);
        self.current_class_stats.insert(stat.to_string(), amount);
    }

    pub fn remove_class_stat(&mut self, stat: &str, amount: u32) {
        let cur = *self
            .current_class_stats
            .get(stat)
            .unwrap_or(&ExtInt::Int(0));
        self.current_class_stats
            .insert(stat.to_string(), cur - amount);
    }

    pub fn add_hp(&mut self, hp: u32, max: i32) {
        let hp = hp as i32;
        self.hp += hp;
        if self.hp > max {
            self.hp = max;
        }
    }

    pub fn add_xp(&mut self, xp: u32, actor: &Rc<Actor>) {
        let factor = Module::rules().experience_factor;
        self.xp += (xp as f32 * factor) as u32;
        self.recompute_level_up(actor);
    }

    pub fn recompute_level_up(&mut self, actor: &Rc<Actor>) {
        self.has_level_up = Module::rules().get_xp_for_next_level(actor.total_level) <= self.xp;
    }

    /// Called on initialization and at the start of a new in game day - resets hp
    /// as well as `end_encounter`, which in turn calls `init_turn`
    pub fn init_day(&mut self, stats: &StatList) {
        self.hp = stats.max_hp;

        for (group, amount) in stats.uses_per_day_iter() {
            self.current_group_uses_per_day
                .insert(group.to_string(), *amount);
        }

        let base_class = self.base_class.as_ref().unwrap();
        for class_stat in base_class.stats.iter() {
            if !class_stat.reset_per_day {
                continue;
            }

            let amount = stats.class_stat_max(&class_stat.id);
            self.current_class_stats
                .insert(class_stat.id.to_string(), amount);
        }

        self.end_encounter(stats);
    }

    /// Called once at the end of each combat encounter - does per encounter
    /// actions as well as `init_turn`
    pub fn end_encounter(&mut self, stats: &StatList) {
        for (group, amount) in stats.uses_per_encounter_iter() {
            self.current_group_uses_per_encounter
                .insert(group.to_string(), *amount);
        }

        let base_class = self.base_class.as_ref().unwrap();
        for class_stat in base_class.stats.iter() {
            if !class_stat.reset_per_encounter {
                continue;
            }

            let amount = stats.class_stat_max(&class_stat.id);
            self.current_class_stats
                .insert(class_stat.id.to_string(), amount);
        }

        self.overflow_ap = 0;
        self.init_turn(stats);
        self.ap = Module::rules().base_ap;
    }

    /// Called each time the entity begins a new turn
    pub fn init_turn(&mut self, stats: &StatList) {
        let rules = Module::rules();

        let mut ap = rules.base_ap as i32 + self.overflow_ap;
        if ap < 0 {
            self.overflow_ap += rules.base_ap as i32;
        } else {
            self.overflow_ap = 0;
        }

        ap += stats.bonus_ap;
        if ap < 0 {
            ap = 0;
        }

        let mut ap = ap as u32;
        if ap > rules.max_ap {
            ap = rules.max_ap;
        }

        self.ap = ap;
    }

    pub fn end_turn(&mut self) {
        let rules = Module::rules();
        let max_overflow = rules.max_overflow_ap;
        self.overflow_ap += self.ap as i32;
        if self.overflow_ap > max_overflow {
            self.overflow_ap = max_overflow;
        }

        self.ap = 0;
    }
}
