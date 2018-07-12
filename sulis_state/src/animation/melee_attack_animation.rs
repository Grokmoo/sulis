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
use std::cell::RefCell;
use std::time::Instant;

use sulis_rules::HitKind;
use sulis_core::util;
use sulis_core::ui::{Color, Widget};
use {script::ScriptEntitySet, ScriptCallback};
use {animation, EntityState, GameState};

pub struct MeleeAttackAnimation {
    attacker: Rc<RefCell<EntityState>>,
    defender: Rc<RefCell<EntityState>>,
    vector: (f32, f32),
    start_time: Instant,
    total_time_millis: u32,
    marked_for_removal: bool,
    has_attacked: bool,
    callback: Option<Box<ScriptCallback>>,

    attack_func: Box<Fn(&Rc<RefCell<EntityState>>, &Rc<RefCell<EntityState>>) ->
        (HitKind, u32, String, Color)>,
}

impl MeleeAttackAnimation {
    pub fn new(attacker: &Rc<RefCell<EntityState>>,
               defender: &Rc<RefCell<EntityState>>,
               total_time_millis: u32,
               attack_func: Box<Fn(&Rc<RefCell<EntityState>>, &Rc<RefCell<EntityState>>) ->
                  (HitKind, u32, String, Color)>) -> MeleeAttackAnimation {

        let x = defender.borrow().location.x + defender.borrow().size.width / 2
            - attacker.borrow().location.x - attacker.borrow().size.width / 2;
        let y = defender.borrow().location.y + defender.borrow().size.height / 2
            - attacker.borrow().location.y - attacker.borrow().size.height / 2;

        MeleeAttackAnimation {
            attacker: Rc::clone(attacker),
            defender: Rc::clone(defender),
            vector: (x as f32, y as f32),
            start_time: Instant::now(),
            total_time_millis,
            marked_for_removal: false,
            has_attacked: false,
            callback: None,
            attack_func,
        }
    }
}

impl animation::Animation for MeleeAttackAnimation {
    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>) {
        self.callback = callback;
    }

    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        if self.marked_for_removal {
            self.attacker.borrow_mut().sub_pos = (0.0, 0.0);
            return false;
        }

        let millis = util::get_elapsed_millis(self.start_time.elapsed());
        let frac = millis as f32 / self.total_time_millis as f32;

        let cb_targets = ScriptEntitySet::new(&self.defender, &vec![Some(Rc::clone(&self.attacker))]);
        if !self.has_attacked && frac > 0.5 {
            if let Some(ref cb) = self.callback.as_ref() {
                cb.before_attack(&cb_targets);
            }

            let area_state = GameState::area_state();

            let defender_cbs = self.defender.borrow().callbacks(&area_state.borrow());
            let attacker_cbs = self.attacker.borrow().callbacks(&area_state.borrow());

            attacker_cbs.iter().for_each(|cb| cb.before_attack(&cb_targets));
            defender_cbs.iter().for_each(|cb| cb.before_defense(&cb_targets));

            let (hit_kind, damage, text, color) = (self.attack_func)(&self.attacker, &self.defender);

            area_state.borrow_mut().add_feedback_text(text, &self.defender, color, 3.0);
            self.has_attacked = true;

            if let Some(ref cb) = self.callback.as_ref() {
                cb.after_attack(&cb_targets, hit_kind, damage);
            }

            attacker_cbs.iter().for_each(|cb| cb.after_attack(&cb_targets, hit_kind, damage));
            defender_cbs.iter().for_each(|cb| cb.after_defense(&cb_targets, hit_kind, damage));
        }

        let mut attacker = self.attacker.borrow_mut();
        if frac > 1.0 {
            attacker.sub_pos = (0.0, 0.0);
            return false;
        } else if frac > 0.5 {
            attacker.sub_pos = ((1.0 - frac) * self.vector.0, (1.0 - frac) * self.vector.1);
        } else {
            attacker.sub_pos = (frac * self.vector.0, frac * self.vector.1);
        }

        true
    }

    fn is_blocking(&self) -> bool { true }

    fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.attacker
    }
}
