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

use {Attribute, Damage, DamageKind, ArmorKind, WeaponKind};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackBonusList {
    pub bonus_damage: Option<Damage>,
    pub accuracy: Option<i32>,
    pub crit_threshold: Option<i32>,
    pub hit_threshold: Option<i32>,
    pub graze_threshold: Option<i32>,
    pub graze_multiplier: Option<f32>,
    pub hit_multiplier: Option<f32>,
    pub crit_multiplier: Option<f32>,
}

impl Default for AttackBonusList {
    fn default() -> AttackBonusList {
        AttackBonusList {
            bonus_damage: None,
            accuracy: None,
            crit_threshold: None,
            hit_threshold: None,
            graze_threshold: None,
            graze_multiplier: None,
            hit_multiplier: None,
            crit_multiplier: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BonusList {
    pub attributes: Option<HashMap<Attribute, i8>>,
    pub ap: Option<i32>,
    pub base_armor: Option<u32>,
    pub armor_kinds: Option<HashMap<DamageKind, u32>>,
    pub bonus_damage: Option<Damage>,
    pub armor_proficiencies: Option<Vec<ArmorKind>>,
    pub weapon_proficiencies: Option<Vec<WeaponKind>>,
    pub bonus_reach: Option<f32>,
    pub bonus_range: Option<f32>,
    pub initiative: Option<i32>,
    pub hit_points: Option<i32>,
    pub accuracy: Option<i32>,
    pub defense: Option<i32>,
    pub fortitude: Option<i32>,
    pub reflex: Option<i32>,
    pub will: Option<i32>,
    pub concealment: Option<i32>,
    pub crit_threshold: Option<i32>,
    pub hit_threshold: Option<i32>,
    pub graze_threshold: Option<i32>,
    pub graze_multiplier: Option<f32>,
    pub hit_multiplier: Option<f32>,
    pub crit_multiplier: Option<f32>,
    pub movement_rate: Option<f32>,
    pub attack_cost: Option<i32>,
    #[serde(default)]
    pub move_disabled: bool,
    #[serde(default)]
    pub attack_disabled: bool,
    #[serde(default)]
    pub group_uses_per_encounter: Vec<(String, u32)>,
}

impl Default for BonusList {
    fn default() -> BonusList {
        BonusList {
            attributes: None,
            ap: None,
            base_armor: None,
            armor_kinds: None,
            armor_proficiencies: None,
            weapon_proficiencies: None,
            bonus_damage: None,
            bonus_range: None,
            bonus_reach: None,
            initiative: None,
            hit_points: None,
            accuracy: None,
            defense: None,
            fortitude: None,
            reflex: None,
            will: None,
            concealment: None,
            crit_threshold: None,
            hit_threshold: None,
            graze_threshold: None,
            graze_multiplier: None,
            hit_multiplier: None,
            crit_multiplier: None,
            movement_rate: None,
            attack_cost: None,
            move_disabled: false,
            attack_disabled: false,
            group_uses_per_encounter: Vec::new(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackBuilder {
    pub damage: Damage,
    pub kind: AttackKindBuilder,
    pub bonuses: AttackBonusList,
}

impl AttackBuilder {
    pub fn distance(&self) -> f32 {
        match self.kind {
            AttackKindBuilder::Melee { reach } => reach,
            AttackKindBuilder::Ranged { range, .. } => range,
        }
    }

    pub fn mult(&mut self, multiplier: f32) -> AttackBuilder {
        AttackBuilder {
            damage: self.damage.mult_f32(multiplier),
            kind: self.kind.clone(),
            bonuses: self.bonuses.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum AttackKindBuilder {
    Melee {
        reach: f32,
    },
    Ranged {
        range: f32,
        projectile: String,
    },
}
