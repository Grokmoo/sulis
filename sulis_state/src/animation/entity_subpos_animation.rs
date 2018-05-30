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

use std::cmp::Ordering;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use sulis_core::ui::{Widget};
use sulis_core::util;

use {animation, EntityState, ScriptCallback};
use animation::particle_generator::Param;

pub struct EntitySubposAnimation {
    owner: Rc<RefCell<EntityState>>,
    position: (Param, Param),
    duration_secs: f32,

    start_time: Instant,
    previous_secs: f32,
    callbacks: Vec<(f32, Box<ScriptCallback>)>, //sorted by the first field which is time in seconds
    callback: Option<Box<ScriptCallback>>,
}

impl EntitySubposAnimation {
    pub fn new(owner: Rc<RefCell<EntityState>>, position: (Param, Param),
        duration_secs: f32) -> EntitySubposAnimation {
        trace!("Created new entity subpos animation with duration {}", duration_secs);
        EntitySubposAnimation {
            owner,
            callback: None,
            callbacks: Vec::new(),
            start_time: Instant::now(),
            previous_secs: 0.0,
            position,
            duration_secs,
        }
    }

    pub fn add_callback(&mut self, callback: Box<ScriptCallback>, time_secs: f32) {
        self.callbacks.push((time_secs, callback));

        self.callbacks.sort_by(|a, b| {
            if a.0 < b.0 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
    }
}

impl animation::Animation for EntitySubposAnimation {
    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        let secs = util::get_elapsed_millis(self.start_time.elapsed()) as f32 / 1000.0;

        let v_term = secs;
        let a_term = secs * secs;
        let j_term = secs * secs * secs;

        self.position.0.update(v_term, a_term, j_term);
        self.position.1.update(v_term, a_term, j_term);

        self.owner.borrow_mut().sub_pos = (self.position.0.value, self.position.1.value);

        if !self.callbacks.is_empty() {
            if secs > self.callbacks[0].0 {
                self.callbacks[0].1.on_anim_update();
                self.callbacks.remove(0);
            }
        }

        self.previous_secs = secs;
        if secs < self.duration_secs {
            true
        } else {
            self.owner.borrow_mut().sub_pos = (0.0, 0.0);
            if let Some(ref mut cb) = self.callback {
                cb.on_anim_complete();
            }
            false
        }
    }

    fn is_blocking(&self) -> bool { true }

    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>) {
        self.callback = callback;
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }
}
