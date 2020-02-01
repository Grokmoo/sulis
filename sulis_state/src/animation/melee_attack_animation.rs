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

use std::cell::RefCell;
use std::rc::Rc;

use crate::{animation::Anim, AreaFeedbackText, EntityState, GameState};
use crate::{script::ScriptEntitySet, ScriptCallback};
use sulis_module::{DamageKind, HitFlags, HitKind};

pub(in crate::animation) fn update(
    attacker: &Rc<RefCell<EntityState>>,
    model: &mut MeleeAttackAnimModel,
    frac: f32,
) {
    if !model.has_attacked && frac > 0.5 {
        let cb_def_targets =
            ScriptEntitySet::new(&model.defender, &vec![Some(Rc::clone(attacker))]);
        let cb_att_targets =
            ScriptEntitySet::new(attacker, &vec![Some(Rc::clone(&model.defender))]);

        for cb in model.callbacks.iter() {
            cb.before_attack(&cb_def_targets);
        }

        let area_state = GameState::area_state();

        let (defender_cbs, attacker_cbs) = {
            let mgr = GameState::turn_manager();
            let mgr = mgr.borrow();
            (
                model.defender.borrow().callbacks(&mgr),
                attacker.borrow().callbacks(&mgr),
            )
        };

        attacker_cbs
            .iter()
            .for_each(|cb| cb.before_attack(&cb_def_targets));
        defender_cbs
            .iter()
            .for_each(|cb| cb.before_defense(&cb_att_targets));

        model.has_attacked = true;
        let result = (model.attack_func)(attacker, &model.defender);
        for entry in result {
            let (hit_kind, hit_flags, damage) = entry;
            let feedback = AreaFeedbackText::with_damage(
                &model.defender.borrow(),
                &area_state.borrow(),
                hit_kind,
                hit_flags,
                &damage,
            );
            area_state.borrow_mut().add_feedback_text(feedback);

            for cb in model.callbacks.iter() {
                cb.after_attack(&cb_def_targets, hit_kind, damage.clone());
            }

            attacker_cbs
                .iter()
                .for_each(|cb| cb.after_attack(&cb_att_targets, hit_kind, damage.clone()));
            defender_cbs
                .iter()
                .for_each(|cb| cb.after_defense(&cb_def_targets, hit_kind, damage.clone()));
        }
    }

    let mut attacker = attacker.borrow_mut();
    if frac > 0.5 {
        attacker.sub_pos = ((1.0 - frac) * model.vector.0, (1.0 - frac) * model.vector.1);
    } else {
        attacker.sub_pos = (frac * model.vector.0, frac * model.vector.1);
    }
}

pub(in crate::animation) fn cleanup(owner: &Rc<RefCell<EntityState>>) {
    owner.borrow_mut().sub_pos = (0.0, 0.0);

    if !GameState::is_combat_active() {
        let area_state = GameState::get_area_state(&owner.borrow().location.area_id).unwrap();
        let mgr = GameState::turn_manager();
        mgr.borrow_mut()
            .check_ai_activation(&owner, &mut area_state.borrow_mut());
    }
}

pub(in crate::animation) struct MeleeAttackAnimModel {
    defender: Rc<RefCell<EntityState>>,
    callbacks: Vec<Box<dyn ScriptCallback>>,
    vector: (f32, f32),
    pub(in crate::animation) has_attacked: bool,
    attack_func: Box<
        dyn Fn(
            &Rc<RefCell<EntityState>>,
            &Rc<RefCell<EntityState>>,
        ) -> Vec<(HitKind, HitFlags, Vec<(DamageKind, u32)>)>,
    >,
}

pub fn new(
    attacker: &Rc<RefCell<EntityState>>,
    defender: &Rc<RefCell<EntityState>>,
    duration_millis: u32,
    callbacks: Vec<Box<dyn ScriptCallback>>,
    attack_func: Box<
        dyn Fn(
            &Rc<RefCell<EntityState>>,
            &Rc<RefCell<EntityState>>,
        ) -> Vec<(HitKind, HitFlags, Vec<(DamageKind, u32)>)>,
    >,
) -> Anim {
    let x = defender.borrow().location.x + defender.borrow().size.width / 2
        - attacker.borrow().location.x
        - attacker.borrow().size.width / 2;
    let y = defender.borrow().location.y + defender.borrow().size.height / 2
        - attacker.borrow().location.y
        - attacker.borrow().size.height / 2;
    let vector = (x as f32, y as f32);

    let model = MeleeAttackAnimModel {
        defender: Rc::clone(defender),
        callbacks,
        has_attacked: false,
        attack_func,
        vector,
    };

    Anim::new_melee_attack(attacker, duration_millis, model)
}
