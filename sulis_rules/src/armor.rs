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

use DamageKind;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    pub base: u32,
    pub kinds: Vec<(DamageKind, u32)>,
}

impl Default for Armor {
    fn default() -> Armor {
        Armor {
            base: 0,
            kinds: Vec::new(),
        }
    }
}

impl Armor {
    pub fn add(&mut self, other: &Armor) {
        self.base += other.base;

        for &(other_kind, other_amount) in other.kinds.iter() {
            let mut found_index: Option<usize> = None;
            for (index, &(this_kind, _)) in self.kinds.iter().enumerate() {
                if other_kind == this_kind {
                    found_index = Some(index);
                    break;
                }
            }

            match found_index {
                Some(index) => self.kinds[index] = (other_kind, self.kinds[index].1 + other_amount),
                None => self.kinds.push((other_kind, other_amount)),
            }
        }
    }

    /// Returns the amount of damage resistance that this armor value
    /// applies to the specified damage kind.
    pub fn amount(&self, check_kind: DamageKind) -> u32 {
        if check_kind == DamageKind::Raw { return 0; }

        for &(kind, amount) in self.kinds.iter() {
            if kind == check_kind { return amount; }
        }

        self.base
    }
}
