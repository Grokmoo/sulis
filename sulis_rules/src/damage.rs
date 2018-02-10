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

use std::fmt::{self, Display};
use rand::{self, Rng};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum DamageKind {
    Slashing,
    Piercing,
    Crushing,
    Acid,
    Cold,
    Electrical,
    Fire,
    Sonic,
    Raw,
}

impl Display for DamageKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct Damage {
    pub min: u32,
    pub max: u32,
    pub kind: DamageKind,
}

impl Damage {
    pub fn max(this: Damage, other: Damage) -> Damage {
        if other.average() > this.average() {
            other
        } else {
            this
        }
    }

    pub fn average(&self) -> f32 {
        (self.min as f32 + self.max as f32) / 2.0
    }

    pub fn roll(&self) -> u32 {
        rand::thread_rng().gen_range(self.min, self.max + 1)
    }
}

impl Default for Damage {
    fn default() -> Damage {
        Damage {
            min: 0,
            max: 0,
            kind: DamageKind::Raw,
        }
    }
}
