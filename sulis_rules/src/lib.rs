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

extern crate rand;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod armor;
pub use self::armor::Armor;

pub mod attribute;
pub use self::attribute::Attribute;

mod bonus_list;
pub use self::bonus_list::BonusList;

pub mod damage;
pub use self::damage::Damage;
pub use self::damage::DamageKind;
pub use self::damage::DamageList;

use self::attribute::Attribute::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitKind {
    Miss,
    Graze,
    Hit,
    Crit,
}

#[derive(Clone)]
pub struct AttributeList([(Attribute, u8); 6]);

impl Default for AttributeList {
    fn default() -> AttributeList {
        AttributeList([
            (Strength, attribute::BASE_VALUE),
            (Dexterity, attribute::BASE_VALUE),
            (Endurance, attribute::BASE_VALUE),
            (Perception, attribute::BASE_VALUE),
            (Intellect, attribute::BASE_VALUE),
            (Wisdom, attribute::BASE_VALUE),
        ])
    }
}

impl AttributeList {
    pub fn get(&self, attr: Attribute) -> u8 {
        match self.0.iter().find(|a| a.0 == attr) {
            Some(val) => val.1,
            None => 0,
        }
    }

    pub fn set(&mut self, attr: Attribute, value: u8) {
        if let Some(attr) = self.0.iter_mut().find(|a| a.0 == attr) {
            attr.1 = value;
        }
    }
}

pub struct StatList {
    bonus_damage: Vec<Damage>,
    bonus_reach: f32,
    base_reach: f32,

    pub damage: DamageList,
    pub armor: Armor,
    pub reach: f32,
    pub max_hp: i32,
    pub initiative: i32,
    pub accuracy: i32,
    pub defense: i32,
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
}

impl StatList {
    /// Adds the bonuses from the specified BonusList to this stat list.
    pub fn add(&mut self, bonuses: &BonusList) {
        self.add_multiple(bonuses, 1);
    }

    /// Adds the specified bonuses to this StatList the specified number of times.
    /// Note that non-numeric bonuses are only added once regardless of the value of
    /// times
    pub fn add_multiple(&mut self, bonuses: &BonusList, times: u32) {
        if times == 0 { return; }

        self.armor.add(bonuses.base_armor, &bonuses.armor_kinds);

        if let Some(bonus_damage) = bonuses.bonus_damage {
            self.bonus_damage.push(bonus_damage.mult(times));
        }

        let times_f32 = times as f32;
        let times_i32 = times as i32;
        if let Some(reach) = bonuses.bonus_reach { self.bonus_reach += reach * times_f32; }
        if let Some(reach) = bonuses.base_reach {
            if reach > self.base_reach {
                self.base_reach = reach;
            }
        }
        if let Some(hit_points) = bonuses.hit_points { self.max_hp += hit_points * times_i32; }
        if let Some(initiative) = bonuses.initiative { self.initiative += initiative * times_i32; }
        if let Some(accuracy) = bonuses.accuracy { self.accuracy += accuracy * times_i32; }
        if let Some(defense) = bonuses.defense { self.defense += defense * times_i32; }
        if let Some(fortitude) = bonuses.fortitude { self.fortitude += fortitude * times_i32; }
        if let Some(reflex) = bonuses.reflex { self.reflex += reflex * times_i32; }
        if let Some(will) = bonuses.will { self.will += will * times_i32; }
    }

    pub fn finalize(&mut self, base_damage: Damage) {
        self.damage.create(base_damage, &self.bonus_damage);

        self.reach = self.bonus_reach + self.base_reach;
    }
}

impl Default for StatList {
    fn default() -> StatList {
        StatList {
            bonus_damage: Vec::new(),
            bonus_reach: 0.0,
            base_reach: 0.0,

            damage: DamageList::new(),
            armor: Armor::default(),
            max_hp: 0,
            reach: 0.0,
            initiative: 0,
            accuracy: 0,
            defense: 0,
            fortitude: 0,
            reflex: 0,
            will: 0,
        }
    }
}
