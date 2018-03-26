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

pub mod bonus_list;
pub use self::bonus_list::BonusList;

pub mod damage;
pub use self::damage::Damage;
pub use self::damage::DamageKind;
pub use self::damage::DamageList;

pub mod stat_list;
pub use self::stat_list::StatList;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum HitKind {
    Miss,
    Graze,
    Hit,
    Crit,
}
