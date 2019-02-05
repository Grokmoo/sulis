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

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Resistance {
    kinds: [i32; 8],
}

impl Default for Resistance {
    fn default() -> Resistance {
        Resistance {
            kinds: [0; 8],
        }
    }
}

impl Resistance {
    pub fn add_kind(&mut self, kind: DamageKind, amount: i32) {
        if kind == DamageKind::Raw { return; }

        let index = kind.index();
        self.kinds[index] = self.kinds[index] + amount;
    }

    /// Returns the amount of damage resistance that this armor value
    /// applies to the specified damage kind.
    pub fn amount(&self, check_kind: DamageKind) -> i32 {
        if check_kind == DamageKind::Raw { return 0; }

        return self.kinds[check_kind.index()]
    }

    pub fn is_empty(&self) -> bool {
        for val in self.kinds.iter() {
            if *val > 0 { return false; }
        }

        true
    }
}
