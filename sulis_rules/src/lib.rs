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

pub mod armor;
pub use self::armor::Armor;

pub mod attribute;
pub use self::attribute::Attribute;

mod bonus_list;
pub use self::bonus_list::BonusList;

pub mod damage;
pub use self::damage::Damage;
pub use self::damage::DamageKind;

use self::attribute::Attribute::*;

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
    pub damage: Damage,
    pub armor: Armor,
    pub reach: f32,
    pub max_hp: i32,
    pub initiative: i32,
    pub accuracy: i32,
    pub dodge: i32,
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
}

impl StatList {
    pub fn add(&mut self, bonuses: &BonusList) {
        if let Some(ref armor) = bonuses.armor {
            self.armor.add(armor);
        }

        if let Some(damage) = bonuses.damage {
            self.damage = Damage::max(self.damage, damage);
        }

        if let Some(reach) = bonuses.reach { self.reach += reach; }
        if let Some(hit_points) = bonuses.hit_points { self.max_hp += hit_points; }
        if let Some(initiative) = bonuses.initiative { self.initiative += initiative; }
        if let Some(accuracy) = bonuses.accuracy { self.accuracy += accuracy; }
        if let Some(dodge) = bonuses.dodge { self.dodge += dodge; }
        if let Some(fortitude) = bonuses.fortitude { self.fortitude += fortitude; }
        if let Some(reflex) = bonuses.reflex { self.reflex += reflex; }
        if let Some(will) = bonuses.will { self.will += will; }
    }
}

impl Default for StatList {
    fn default() -> StatList {
        StatList {
            damage: Damage::default(),
            armor: Armor::default(),
            max_hp: 0,
            reach: 0.0,
            initiative: 0,
            accuracy: 0,
            dodge: 0,
            fortitude: 0,
            reflex: 0,
            will: 0,
        }
    }
}
