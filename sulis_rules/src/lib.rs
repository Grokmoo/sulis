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

extern crate sulis_core;

extern crate rand;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod armor;
pub use self::armor::Armor;

pub mod attack;
pub use self::attack::Attack;
pub use self::attack::AttackKind;

pub mod attribute;
pub use self::attribute::Attribute;
pub use self::attribute::AttributeList;

pub mod bonus;
pub use self::bonus::Bonus;
pub use self::bonus::BonusKind;
pub use self::bonus::BonusList;
pub use self::bonus::AttackBonuses;

pub mod damage;
pub use self::damage::Damage;
pub use self::damage::DamageKind;
pub use self::damage::DamageList;

pub mod stat_list;
pub use self::stat_list::StatList;

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Slot {
    Cloak,
    Head,
    Torso,
    Hands,
    HeldMain,
    HeldOff,
    Legs,
    Feet,
    Waist,
    Neck,
    FingerMain,
    FingerOff,
}

impl Slot {
    pub fn iter() -> ::std::slice::Iter<'static, Slot> {
        SLOTS_LIST.iter()
    }
}

use self::Slot::*;

// The sort order of this list is important
const SLOTS_LIST: [Slot; 12] = [Cloak, Feet, Legs, Torso, Hands, Head, HeldMain, HeldOff, Waist,
                                Neck, FingerMain, FingerOff];

#[derive(Debug, Deserialize)]
pub enum ItemKind {
    Armor { kind: ArmorKind },
    Weapon { kind: WeaponKind },
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum HitKind {
    Miss,
    Graze,
    Hit,
    Crit,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deserialize)]
pub enum WeaponKind {
    Axe,
    Crossbow,
    Bow,
    SmallSword,
    LargeSword,
    Hammer,
    Spear,
    Mace,
    Simple,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deserialize)]
pub enum ArmorKind {
    Light,
    Medium,
    Heavy,
}
