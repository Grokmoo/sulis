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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use state::{AreaState, EntityState};
use animation;

/// an animation that does nothing, but causes the AI to wait for a
/// specified time
pub struct WaitAnimation {
    owner: Rc<RefCell<EntityState>>,
    start_time: Instant,
    duration: u32,
}

impl WaitAnimation {
    pub fn new(owner: &Rc<RefCell<EntityState>>, duration: u32) -> WaitAnimation {
        WaitAnimation {
            owner: Rc::clone(owner),
            start_time: Instant::now(),
            duration,
        }
    }
}

impl animation::Animation for WaitAnimation {
    fn update(&mut self, _: &mut AreaState) -> bool {
        animation::get_elapsed_millis(self.start_time.elapsed()) < self.duration
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }
}
