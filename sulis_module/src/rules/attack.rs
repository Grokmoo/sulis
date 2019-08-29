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

use crate::rules::bonus::{AttackBuilder, AttackKindBuilder, BonusKind, BonusList};
use crate::rules::{AttackBonuses, Damage, DamageKind, DamageList, StatList, WeaponKind};
use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;

use crate::rules::AttackKind::*;

fn add_bonus(bonuses: &mut AttackBonuses, bonus_damage: &mut Vec<Damage>, bonus_kind: &BonusKind) {
    match bonus_kind {
        BonusKind::Damage(damage) => bonus_damage.push(damage.clone()),
        BonusKind::MeleeAccuracy(amount) => bonuses.melee_accuracy += amount,
        BonusKind::RangedAccuracy(amount) => bonuses.ranged_accuracy += amount,
        BonusKind::SpellAccuracy(amount) => bonuses.spell_accuracy += amount,
        BonusKind::CritChance(amount) => bonuses.crit_chance -= amount,
        BonusKind::HitThreshold(amount) => bonuses.hit_threshold -= amount,
        BonusKind::GrazeThreshold(amount) => bonuses.graze_threshold -= amount,
        BonusKind::CritMultiplier(amount) => bonuses.crit_multiplier += amount,
        BonusKind::HitMultiplier(amount) => bonuses.hit_multiplier += amount,
        BonusKind::GrazeMultiplier(amount) => bonuses.graze_multiplier += amount,
        _ => {
            warn!(
                "Attack bonus of type '{:?}' will never be applied",
                bonus_kind
            );
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
    pub fn special(
        stats: &StatList,
        min_damage: u32,
        max_damage: u32,
        ap: u32,
        damage_kind: DamageKind,
        attack_kind: AttackKind,
    ) -> Attack {
        let damage = Damage {
            min: min_damage,
            max: max_damage,
            ap,
            kind: Some(damage_kind),
        };

        let mut bonuses = AttackBonuses::default();
        let mut bonus_damage = Vec::new();

        for bonus in stats.attack_bonuses.iter() {
            use crate::rules::bonus::Contingent::*;
            match bonus.when {
                AttackWithWeapon(_) => (), // doesn't apply to this non weapon attack
                AttackWhenHidden => {
                    if stats.hidden {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                }
                AttackWithDamageKind(kind_to_match) => {
                    if damage_kind == kind_to_match {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                }
                _ => unreachable!(),
            }
        }

        let damage_list = DamageList::new(damage, &bonus_damage);

        Attack {
            damage: damage_list,
            kind: attack_kind,
            bonuses,
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
            use crate::rules::bonus::Contingent::*;
            match bonus.when {
                AttackWithWeapon(bonus_weapon_kind) => {
                    if bonus_weapon_kind == weapon_kind {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                }
                AttackWhenHidden => {
                    if stats.hidden {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                }
                AttackWithDamageKind(damage_kind) => {
                    assert!(builder.damage.kind.is_some());
                    if builder.damage.kind.unwrap() == damage_kind {
                        add_bonus(&mut bonuses, &mut bonus_damage, &bonus.kind);
                    }
                }
                _ => unreachable!(),
            }
        }

        let damage = DamageList::new(builder.damage, &bonus_damage);

        let kind = match builder.kind {
            AttackKindBuilder::Melee { reach } => Melee {
                reach: reach + stats.bonus_reach,
            },
            AttackKindBuilder::Ranged {
                range,
                ref projectile,
            } => {
                let projectile = match ResourceSet::image(projectile) {
                    None => {
                        warn!("No image found for projectile '{}'", projectile);
                        "empty".to_string()
                    }
                    Some(_) => projectile.to_string(),
                };
                Ranged {
                    range: range + stats.bonus_range,
                    projectile,
                }
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

    pub fn get_ranged_projectile(&self) -> Option<Rc<dyn Image>> {
        match self.kind {
            Ranged { ref projectile, .. } => ResourceSet::image(projectile),
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
    pub fn from_str(attack: &str, kind: &str) -> AttackKind {
        let kind = match kind {
            "Melee" => AccuracyKind::Melee,
            "Ranged" => AccuracyKind::Ranged,
            "Spell" => AccuracyKind::Spell,
            _ => {
                warn!("Unable to parse string '{}' into accuracy kind", kind);
                AccuracyKind::Melee
            }
        };

        match attack {
            "Fortitude" => AttackKind::Fortitude { accuracy: kind },
            "Reflex" => AttackKind::Reflex { accuracy: kind },
            "Will" => AttackKind::Will { accuracy: kind },
            "Dummy" => AttackKind::Dummy,
            _ => {
                warn!("Unable to parse string '{}' into attack kind", attack);
                AttackKind::Dummy
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttackKind {
    Melee { reach: f32 },
    Ranged { range: f32, projectile: String },
    Fortitude { accuracy: AccuracyKind },
    Reflex { accuracy: AccuracyKind },
    Will { accuracy: AccuracyKind },
    Dummy,
}

#[derive(Debug, Clone, Copy)]
pub enum AccuracyKind {
    Melee,
    Ranged,
    Spell,
}
