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

use sulis_core::util;
use {animation, AreaState, EntityState};

pub struct AttackAnimation {
    attacker: Rc<RefCell<EntityState>>,
    defender: Rc<RefCell<EntityState>>,
    vector: (f32, f32),
    start_time: Instant,
    total_time_millis: u32,
    marked_for_removal: bool,
    has_attacked: bool,
}

impl AttackAnimation {
    pub fn new(attacker: &Rc<RefCell<EntityState>>, defender: &Rc<RefCell<EntityState>>,
               total_time_millis: u32) -> AttackAnimation {
        let x = defender.borrow().location.x + defender.borrow().size.size / 2
            - attacker.borrow().location.x - attacker.borrow().size.size / 2;
        let y = defender.borrow().location.y + defender.borrow().size.size / 2
            - attacker.borrow().location.y - attacker.borrow().size.size / 2;

        AttackAnimation {
            attacker: Rc::clone(attacker),
            defender: Rc::clone(defender),
            vector: (x as f32, y as f32),
            start_time: Instant::now(),
            total_time_millis,
            marked_for_removal: false,
            has_attacked: false,
        }
    }
}

impl animation::Animation for AttackAnimation {
    fn update(&mut self, _area_state: &mut AreaState) -> bool {
        if self.marked_for_removal {
            self.attacker.borrow_mut().sub_pos = (0.0, 0.0);
            return false;
        }

        let millis = util::get_elapsed_millis(self.start_time.elapsed());
        let frac = millis as f32 / self.total_time_millis as f32;
        let mut attacker = self.attacker.borrow_mut();

        if !self.has_attacked && frac > 0.5 {
            attacker.actor.attack(&self.defender);
            self.has_attacked = true;
        }

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

    fn check(&mut self, entity: &Rc<RefCell<EntityState>>) {
        if *self.attacker.borrow() == *entity.borrow() {
            self.marked_for_removal = true;
        }
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.attacker
    }
}
