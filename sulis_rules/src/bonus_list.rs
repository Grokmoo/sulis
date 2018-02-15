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

use {AttackKind, Damage, DamageKind};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BonusList {
    pub attack: Option<AttackBuilder>,
    pub base_armor: Option<u32>,
    pub armor_kinds: Option<HashMap<DamageKind, u32>>,
    pub bonus_damage: Option<Damage>,
    pub bonus_reach: Option<f32>,
    pub bonus_range: Option<f32>,
    pub initiative: Option<i32>,
    pub hit_points: Option<i32>,
    pub accuracy: Option<i32>,
    pub defense: Option<i32>,
    pub fortitude: Option<i32>,
    pub reflex: Option<i32>,
    pub will: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackBuilder {
    pub damage: Damage,
    pub kind: AttackKind,
}

impl AttackBuilder {
    pub fn distance(&self) -> f32 {
        match self.kind {
            AttackKind::Melee { reach } => reach,
            AttackKind::Ranged { range } => range,
        }
    }

    pub fn mult(&mut self, multiplier: f32) -> AttackBuilder {
        AttackBuilder {
            damage: self.damage.mult_f32(multiplier),
            kind: self.kind,
        }
    }
}
