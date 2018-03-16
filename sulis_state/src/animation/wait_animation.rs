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
use std::time::Instant;

use sulis_core::util;
use sulis_core::ui::Widget;
use {animation, EntityState, ScriptCallback};

/// an animation that does nothing, but causes the AI to wait for a
/// specified time
pub struct WaitAnimation {
    owner: Rc<RefCell<EntityState>>,
    start_time: Instant,
    duration: u32,
    callback: Option<Box<ScriptCallback>>,
}

impl WaitAnimation {
    pub fn new(owner: &Rc<RefCell<EntityState>>, duration: u32) -> WaitAnimation {
        WaitAnimation {
            owner: Rc::clone(owner),
            start_time: Instant::now(),
            duration,
            callback: None,
        }
    }
}

impl animation::Animation for WaitAnimation {
    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>) {
        self.callback = callback;
    }

    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        util::get_elapsed_millis(self.start_time.elapsed()) < self.duration
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }
}
