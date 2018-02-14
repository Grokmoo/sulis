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

use DamageKind;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    base: u32,
    kinds: [u32; 8],
}

impl Default for Armor {
    fn default() -> Armor {
        Armor {
            base: 0,
            kinds: [0; 8],
        }
    }
}

impl Armor {
    pub fn add(&mut self, base_armor: Option<u32>, kinds: &Option<HashMap<DamageKind, u32>>) {
        let other_base = match base_armor {
            None => 0,
            Some(base) => base,
        };

        let kinds = match kinds {
            &None => {
                self.kinds.iter_mut().for_each(|amount| *amount += other_base);
                self.base += other_base;
                return;
            },
            &Some(ref kinds) => kinds,
        };

        for kind in DamageKind::iter() {
            if *kind == DamageKind::Raw { continue; }
            let amount = kinds.get(kind).unwrap_or(&other_base);
            self.kinds[kind.index()] += amount;
        }

        self.base += other_base;
    }

    /// Returns the amount of damage resistance that this armor value
    /// applies to the specified damage kind.
    pub fn amount(&self, check_kind: DamageKind) -> u32 {
        if check_kind == DamageKind::Raw { return 0; }

        return self.kinds[check_kind.index()]
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    pub fn differs_from_base(&self, kind: DamageKind) -> bool {
        if kind == DamageKind::Raw { return true; }

        return self.kinds[kind.index()] != self.base
    }
}
