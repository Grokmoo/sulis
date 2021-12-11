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

use crate::rules::DamageKind;

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    base: i32,
    kinds: [i32; 7],
}

impl Armor {
    pub fn finalize(&mut self) {
        if self.base < 0 {
            self.base = 0;
        }

        for index in 0..self.kinds.len() {
            if self.kinds[index] < 0 {
                self.kinds[index] = 0;
            }
        }
    }

    pub fn add_base(&mut self, amount: i32) {
        self.base += amount;

        for index in 0..self.kinds.len() {
            self.kinds[index] += amount;
        }
    }

    pub fn add_kind(&mut self, kind: DamageKind, amount: i32) {
        if kind == DamageKind::Raw {
            return;
        }

        let index = kind.index();
        self.kinds[index] += amount;
    }

    /// Returns the amount of armor that this Armor value
    /// applies to the specified damage kind.
    pub fn amount(&self, check_kind: DamageKind) -> i32 {
        if check_kind == DamageKind::Raw {
            return 0;
        }

        self.kinds[check_kind.index()]
    }

    pub fn base(&self) -> i32 {
        self.base
    }

    pub fn is_empty(&self) -> bool {
        if self.base != 0 {
            return false;
        }

        for val in self.kinds.iter() {
            if *val != 0 {
                return false;
            }
        }

        true
    }

    pub fn differs_from_base(&self, kind: DamageKind) -> bool {
        if kind == DamageKind::Raw {
            return true;
        }

        self.kinds[kind.index()] != self.base
    }
}
