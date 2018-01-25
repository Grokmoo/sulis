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

use {animation, AreaState, EntityState};
use sulis_core::util::Point;

pub struct MoveAnimation {
   mover: Rc<RefCell<EntityState>>,
   path: Vec<Point>,
   last_frame_index: i32,
   start_time: Instant,
   frame_time_millis: u32,
   marked_for_removal: bool,
}

impl MoveAnimation {
    pub fn new(mover: Rc<RefCell<EntityState>>,
               path: Vec<Point>, frame_time_millis: u32) -> MoveAnimation {

        MoveAnimation {
            mover,
            path,
            start_time: Instant::now(),
            frame_time_millis,
            marked_for_removal: false,
            last_frame_index: -1,
        }
    }

}

impl animation::Animation for MoveAnimation {
    fn update(&mut self, area_state: &mut AreaState) -> bool {
        if self.marked_for_removal || self.path.is_empty() { return false; }

        let frame_index = animation::get_current_frame(self.start_time.elapsed(),
            self.frame_time_millis, self.path.len() - 1);

        if frame_index as i32 == self.last_frame_index {
            return true;
        }
        let move_ap = frame_index as i32 - self.last_frame_index;
        self.last_frame_index = frame_index as i32;

        let p = self.path[frame_index];
        if !area_state.move_entity(&self.mover, p.x, p.y, move_ap as u32) {
            return false;
        }

        trace!("Updated move animation at frame {}", frame_index);
        return frame_index != self.path.len() - 1
    }

    fn check(&mut self, entity: &Rc<RefCell<EntityState>>) {
        if self.mover.borrow().index == entity.borrow().index {
            self.marked_for_removal = true;
        }
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.mover
    }
}
