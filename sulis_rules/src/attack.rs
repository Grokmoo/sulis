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

use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use bonus::{AttackBuilder, AttackKindBuilder, BonusKind, BonusList};
use {Armor, AttackBonuses, Damage, DamageKind, DamageList, StatList, WeaponKind};

use AttackKind::*;

fn add_bonus(bonuses: &mut AttackBonuses, bonus_damage: &mut Vec<Damage>, bonus_kind: &BonusKind) {
    match bonus_kind{
        BonusKind::Damage(damage) => bonus_damage.push(damage.clone()),
        BonusKind::Accuracy(amount) => bonuses.accuracy += amount,
        BonusKind::CritThreshold(amount) => bonuses.crit_threshold += amount,
        BonusKind::HitThreshold(amount) => bonuses.hit_threshold += amount,
        BonusKind::GrazeThreshold(amount) => bonuses.graze_threshold += amount,
        BonusKind::CritMultiplier(amount) => bonuses.crit_multiplier += amount,
        BonusKind::HitMultiplier(amount) => bonuses.hit_multiplier += amount,
        BonusKind::GrazeMultiplier(amount) => bonuses.graze_multiplier += amount,
        _ => {
            warn!("Attack bonus of type '{:?}' will never be applied", bonus_kind);
        }
    }
}

#[derive(Clone)]
pub struct Attack {
    pub damage: DamageList,
    pub kind: AttackKind,
    pub bonuses: AttackBonuses,
}

impl Attack {
    pub fn special(min_damage: u32, max_damage: u32, ap: u32,
                   damage_kind: DamageKind, attack_kind: AttackKind) -> Attack {
        let damage = Damage {
            min: min_damage,
            max: max_damage,
            ap: ap,
            kind: Some(damage_kind),
        };

        let damage_list = DamageList::from(damage);

        Attack {
            damage: damage_list,
            kind: attack_kind,
            bonuses: AttackBonuses::default(),
        }
    }

    pub fn from(other: &Attack, bonuses_to_add: &BonusList) -> Attack {
        let kind = other.kind.clone();
        let mut bonuses = other.bonuses.clone();
        let mut bonus_damage = other.damage.clone().to_vec();
        assert!(bonus_damage.len() > 0);

        let base_damage = bonus_damage.remove(0);

        for bonus in bonuses_to_add.iter() {
            add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
        }

        let damage = DamageList::new(base_damage, &bonus_damage);

        Attack {
            kind,
            bonuses,
            damage,
        }
    }

    pub fn new(builder: &AttackBuilder, stats: &StatList, weapon_kind: WeaponKind) -> Attack {
        let mut bonuses = builder.bonuses.clone();
        let mut bonus_damage = stats.bonus_damage.clone();
        if let Some(damage) = bonuses.damage {
            bonus_damage.push(damage);
        }
        for bonus in stats.attack_bonuses.iter() {
            use bonus::Contingent::*;
            match bonus.when {
                AttackWithWeapon(bonus_weapon_kind) => {
                    if bonus_weapon_kind == weapon_kind {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                },
                AttackWhenHidden => {
                    if stats.hidden {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                },
                _ => unreachable!(),
            }
        }

        let damage = DamageList::new(builder.damage, &bonus_damage);

        let kind = match builder.kind {
            AttackKindBuilder::Melee { reach } =>
                Melee { reach: reach + stats.bonus_reach },
            AttackKindBuilder::Ranged { range, ref projectile } => {
                let projectile = match ResourceSet::get_image(projectile) {
                    None => {
                        warn!("No image found for projectile '{}'", projectile);
                        "empty".to_string()
                    }, Some(_) => projectile.to_string(),
                };
                Ranged { range: range + stats.bonus_range, projectile }
            }
        };

        Attack {
            damage,
            kind,
            bonuses,
        }
    }

    pub fn is_melee(&self) -> bool {
        match self.kind {
            Melee { .. } => true,
            _ => false,
        }
    }

    pub fn is_ranged(&self) -> bool {
        match self.kind {
            Ranged { .. } => true,
            _ => false,
        }
    }

    pub fn get_ranged_projectile(&self) -> Option<Rc<Image>> {
        match self.kind {
            Ranged { ref projectile, .. } => {
                ResourceSet::get_image(projectile)
            },
            _ => None,
        }
    }

    pub fn mult(&self, multiplier: f32) -> Attack {
        Attack {
            damage: self.damage.mult(multiplier),
            kind: self.kind.clone(),
            bonuses: self.bonuses.clone(),
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
            Ranged { range, .. } => range,
            _ => 0.0,
        }
    }
}

impl AttackKind {
    pub fn from_str(s: &str) -> AttackKind {
        match s {
            "Fortitude" => AttackKind::Fortitude,
            "Reflex" => AttackKind::Reflex,
            "Will" => AttackKind::Will,
            _ => {
                warn!("Unable to parse string '{}' into attack kind", s);
                AttackKind::Will
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttackKind {
    Melee { reach: f32 },
    Ranged { range: f32, projectile: String },
    Fortitude,
    Reflex,
    Will,
}
