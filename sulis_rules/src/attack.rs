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

use bonus_list::AttackBuilder;
use {Armor, DamageKind, DamageList, StatList};

use AttackKind::*;

pub struct Attack {
    pub damage: DamageList,
    pub kind: AttackKind,
}

impl Attack {
    pub fn new(builder: &AttackBuilder, stats: &StatList) -> Attack {
        let damage = DamageList::new(builder.damage, &stats.bonus_damage);

        let kind = match builder.kind {
            Melee { reach } => Melee { reach: reach + stats.bonus_reach },
            Ranged { range } => Ranged { range: range + stats.bonus_range },
        };

        Attack {
            damage,
            kind,
        }
    }

    pub fn mult(&self, multiplier: f32) -> Attack {
        Attack {
            damage: self.damage.mult(multiplier),
            kind: self.kind,
        }
    }

    pub fn roll_damage(&self, armor: &Armor,
                       multiplier: f32) -> Vec<(DamageKind, u32)> {
        self.damage.roll(armor, multiplier)
    }

    // Returns the distance that this attack can reach
    pub fn distance(&self) -> f32 {
        match self.kind {
            Melee { reach } => reach,
            Ranged { range } => range,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields, untagged)]
pub enum AttackKind {
    Melee { reach: f32 },
    Ranged { range: f32 },
}
