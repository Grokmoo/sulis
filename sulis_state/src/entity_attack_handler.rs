//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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


use crate::{center, is_threat, ActorState, EntityState, GameState, RcRfc};
use sulis_module::{AccuracyKind, Attack, AttackKind, DamageKind, HitFlags, HitKind, Module};

fn is_sneak_attack(parent: &EntityState, target: &EntityState) -> bool {
    parent.actor.stats.hidden && !target.actor.stats.sneak_attack_immunity
}

fn is_flanking(parent: &EntityState, target: &EntityState) -> bool {
    if target.actor.stats.flanked_immunity {
        return false;
    }

    let mgr = GameState::turn_manager();
    let area = GameState::get_area_state(&parent.location.area_id).unwrap();
    let area = area.borrow();
    for entity_index in area.entity_iter() {
        if *entity_index == parent.index() || *entity_index == target.index() {
            continue;
        }

        let entity = mgr.borrow().entity(*entity_index);
        let entity = entity.borrow();

        if !is_threat(&entity, target) {
            continue;
        }

        let p_target = center(target);
        let p_parent = center(parent);
        let p_other = center(&*entity);

        let p1 = (p_target.0 - p_parent.0, p_target.1 - p_parent.1);
        let p2 = (p_target.0 - p_other.0, p_target.1 - p_other.1);

        let mut cos_angle = (p1.0 * p2.0 + p1.1 * p2.1) / (p1.0.hypot(p1.1) * p2.0.hypot(p2.1));
        if cos_angle > 1.0 {
            cos_angle = 1.0;
        }
        if cos_angle < -1.0 {
            cos_angle = -1.0;
        }

        let angle = cos_angle.acos().to_degrees();

        debug!(
            "Got angle {} between {} and {} attacking {}",
            angle, parent.actor.actor.name, entity.actor.actor.name, target.actor.actor.name
        );

        if angle > parent.actor.stats.flanking_angle as f32 {
            return true;
        }
    }

    false
}

pub fn weapon_attack(
    parent: &RcRfc<EntityState>,
    target: &RcRfc<EntityState>,
) -> Vec<(HitKind, HitFlags, Vec<(DamageKind, u32)>)> {
    if target.borrow_mut().actor.hp() <= 0 {
        return vec![(HitKind::Miss, HitFlags::default(), Vec::new())];
    }

    info!(
        "'{}' attacks '{}'",
        parent.borrow().actor.actor.name,
        target.borrow().actor.actor.name
    );

    let attacks = parent.borrow().actor.stats.attacks.clone();

    let is_flanking = is_flanking(&parent.borrow(), &target.borrow());
    let is_sneak_attack = is_sneak_attack(&parent.borrow(), &target.borrow());

    let mut result = Vec::new();
    for attack in attacks {
        let mut attack = if is_flanking {
            Attack::from(&attack, &parent.borrow().actor.stats.flanking_bonuses)
        } else {
            attack
        };

        let (hit_kind, hit_flags, damage) =
            attack_internal(parent, target, &mut attack, is_flanking, is_sneak_attack);
        result.push((hit_kind, hit_flags, damage));
    }

    ActorState::check_death(parent, target);
    result
}

pub fn attack(
    parent: &RcRfc<EntityState>,
    target: &RcRfc<EntityState>,
    attack: &mut Attack,
) -> (HitKind, HitFlags, Vec<(DamageKind, u32)>) {
    if target.borrow_mut().actor.hp() <= 0 {
        return (HitKind::Miss, HitFlags::default(), Vec::new());
    }

    info!(
        "'{}' attacks '{}'",
        parent.borrow().actor.actor.name,
        target.borrow().actor.actor.name
    );

    let is_flanking = is_flanking(&parent.borrow(), &target.borrow());
    let is_sneak_attack = is_sneak_attack(&parent.borrow(), &target.borrow());

    let (hit_kind, hit_flags, damage) =
        attack_internal(parent, target, attack, is_flanking, is_sneak_attack);

    ActorState::check_death(parent, target);

    (hit_kind, hit_flags, damage)
}

fn attack_internal(
    parent: &RcRfc<EntityState>,
    target: &RcRfc<EntityState>,
    attack: &mut Attack,
    flanking: bool,
    sneak_attack: bool,
) -> (HitKind, HitFlags, Vec<(DamageKind, u32)>) {
    let rules = Module::rules();

    let concealment = std::cmp::max(
        0,
        target.borrow().actor.stats.concealment - parent.borrow().actor.stats.concealment_ignore,
    );

    if !rules.concealment_roll(concealment) {
        debug!("Concealment miss");
        return (
            HitKind::Miss,
            HitFlags {
                concealment: true,
                ..Default::default()
            },
            Vec::new(),
        );
    }

    let (accuracy_kind, defense) = {
        let target_stats = &target.borrow().actor.stats;
        match attack.kind {
            AttackKind::Fortitude { accuracy } => (accuracy, target_stats.fortitude),
            AttackKind::Reflex { accuracy } => (accuracy, target_stats.reflex),
            AttackKind::Will { accuracy } => (accuracy, target_stats.will),
            AttackKind::Melee { .. } => (AccuracyKind::Melee, target_stats.defense),
            AttackKind::Ranged { .. } => (AccuracyKind::Ranged, target_stats.defense),
            AttackKind::Dummy => {
                return (HitKind::Hit, HitFlags::default(), Vec::new());
            }
        }
    };
    let crit_immunity = target.borrow().actor.stats.crit_immunity;

    if flanking {
        attack.bonuses.melee_accuracy += rules.flanking_accuracy_bonus;
        attack.bonuses.ranged_accuracy += rules.flanking_accuracy_bonus;
        attack.bonuses.spell_accuracy += rules.flanking_accuracy_bonus;
    } else if sneak_attack {
        attack.bonuses.melee_accuracy += rules.hidden_accuracy_bonus;
        attack.bonuses.ranged_accuracy += rules.hidden_accuracy_bonus;
        attack.bonuses.spell_accuracy += rules.hidden_accuracy_bonus;
    }

    let hit_flags = HitFlags {
        flanking,
        sneak_attack,
        concealment: false,
    };

    let (hit_kind, damage_multiplier) = {
        let parent_stats = &parent.borrow().actor.stats;
        let hit_kind =
            parent_stats.attack_roll(accuracy_kind, crit_immunity, defense, &attack.bonuses);
        let damage_multiplier = match hit_kind {
            HitKind::Miss => {
                debug!("Miss");
                return (HitKind::Miss, hit_flags, Vec::new());
            }
            HitKind::Graze => parent_stats.graze_multiplier + attack.bonuses.graze_multiplier,
            HitKind::Hit => parent_stats.hit_multiplier + attack.bonuses.hit_multiplier,
            HitKind::Crit => parent_stats.crit_multiplier + attack.bonuses.crit_multiplier,
            HitKind::Auto => panic!(),
        };
        (hit_kind, damage_multiplier)
    };

    let damage = {
        let target = &target.borrow().actor.stats;
        let damage = &attack.damage;
        rules.roll_damage(damage, &target.armor, &target.resistance, damage_multiplier)
    };

    debug!("{:?}. {:?} damage", hit_kind, damage);

    if !damage.is_empty() {
        let mut total = 0;
        for (_kind, amount) in damage.iter() {
            total += amount;
        }

        EntityState::remove_hp(target, parent, hit_kind, damage.clone());
    }

    (hit_kind, hit_flags, damage)
}
