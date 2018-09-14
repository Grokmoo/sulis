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

use std::str::FromStr;
use std::io::{Error, ErrorKind};

extern crate sulis_core;

extern crate rand;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod armor;
pub use self::armor::Armor;

pub mod attack;
pub use self::attack::Attack;
pub use self::attack::AttackKind;
pub use self::attack::AccuracyKind;

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

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl FromStr for Slot {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s {
            "cloak" => Slot::Cloak,
            "head" => Slot::Head,
            "torso" => Slot::Torso,
            "hands" => Slot::Hands,
            "held_main" => Slot::HeldMain,
            "held_off" => Slot::HeldOff,
            "legs" => Slot::Legs,
            "feet" => Slot::Feet,
            "waist" => Slot::Waist,
            "neck" => Slot::Neck,
            "finger_main" => Slot::FingerMain,
            "finger_off" => Slot::FingerOff,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unable to parse Slot from '{}'", s)));
            },
        };

        Ok(val)
    }
}

use self::Slot::*;

// The sort order of this list is important
const SLOTS_LIST: [Slot; 12] = [Cloak, Feet, Legs, Torso, Hands, Head, HeldMain, HeldOff, Waist,
                                Neck, FingerMain, FingerOff];

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum QuickSlot {
    AltHeldMain,
    AltHeldOff,
    Usable1,
    Usable2,
    Usable3,
    Usable4,
}

impl QuickSlot {
    pub fn iter() -> ::std::slice::Iter<'static, QuickSlot> {
        QUICKSLOTS_LIST.iter()
    }

    pub fn usable_iter() -> ::std::slice::Iter<'static, QuickSlot> {
        USABLE_QUICKSLOTS_LIST.iter()
    }
}

use self::QuickSlot::*;

const QUICKSLOTS_LIST: [QuickSlot; 6] = [ AltHeldMain, AltHeldOff, Usable1, Usable2, Usable3,
                                          Usable4];

const USABLE_QUICKSLOTS_LIST: [QuickSlot; 4] = [ Usable1, Usable2, Usable3, Usable4 ];

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl FromStr for HitKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s {
            "miss" => HitKind::Miss,
            "graze" => HitKind::Graze,
            "hit" => HitKind::Hit,
            "crit" => HitKind::Crit,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unable to parse HitKind from '{}'", s)));
            },
        };

        Ok(val)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum WeaponStyle {
    Ranged,
    TwoHanded,
    Single,
    Shielded,
    DualWielding,
}

impl FromStr for WeaponStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s {
            "ranged" => WeaponStyle::Ranged,
            "two_handed" => WeaponStyle::TwoHanded,
            "single" => WeaponStyle::Single,
            "shielded" => WeaponStyle::Shielded,
            "dual_wielding" => WeaponStyle::DualWielding,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unable to parse WeaponStyle from '{}'", s)));
            },
        };

        Ok(val)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
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

impl FromStr for WeaponKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s {
            "axe" => WeaponKind::Axe,
            "crossbow" => WeaponKind::Crossbow,
            "bow" => WeaponKind::Bow,
            "small_sword" => WeaponKind::SmallSword,
            "large_sword" => WeaponKind::LargeSword,
            "hammer" => WeaponKind::Hammer,
            "spear" => WeaponKind::Spear,
            "mace" => WeaponKind::Mace,
            "simple" => WeaponKind::Simple,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unable to parse WeaponKind from '{}'", s)));
            },
        };

        Ok(val)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ArmorKind {
    Light,
    Medium,
    Heavy,
}

impl FromStr for ArmorKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s {
            "light" => ArmorKind::Light,
            "medium" => ArmorKind::Medium,
            "heavy" => ArmorKind::Heavy,
            _ => {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unable to parse ArmorKind from '{}'", s)));
            },
        };

        Ok(val)
    }
}
