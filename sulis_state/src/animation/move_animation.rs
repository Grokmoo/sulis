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

use std::cmp;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use {animation, EntityState, GameState, ScriptCallback};
use sulis_core::ui::Widget;
use sulis_core::util::{self, Point};

pub struct MoveAnimation {
   mover: Rc<RefCell<EntityState>>,
   path: Vec<Point>,
   last_frame_index: i32,
   start_time: Instant,
   frame_time_millis: u32,
   marked_for_removal: bool,
   callback: Option<Box<ScriptCallback>>,

   smoothed_path: Vec<(f32, f32)>,
}

impl MoveAnimation {
    pub fn new(mover: Rc<RefCell<EntityState>>,
               path: Vec<Point>, frame_time_millis: u32) -> MoveAnimation {

        let mut smoothed_path = Vec::new();
        let mut prev2 = path[0];
        let mut prev = path[0];
        let mut next = path[path.len() - 1];
        let mut next2 = path[path.len() - 1];

        for i in 0..(path.len() as i32) {
            let cur: Point = path[i as usize];
            if i < path.len() as i32 - 2 {
                next = path[i as usize + 1];
                next2 = path[i as usize + 2];
            } if i < path.len() as i32 - 1 {
                next = path[i as usize + 1];
                // next2 is already set to the final point
            } else {
                // next and next2 are already set to the final point
            }

            let mut avg_x = (prev2.x + prev.x + cur.x + next.x + next2.x) as f32 / 5.0;
            let mut avg_y = (prev2.y + prev.y + cur.y + next.y + next2.y) as f32 / 5.0;

            smoothed_path.push((avg_x, avg_y));

            prev2 = prev;
            prev = cur;
        }

        MoveAnimation {
            mover,
            path,
            start_time: Instant::now(),
            frame_time_millis,
            marked_for_removal: false,
            last_frame_index: 0, // start at index 0 which is the initial pos
            callback: None,
            smoothed_path,
        }
    }

}

impl animation::Animation for MoveAnimation {
    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>) {
        self.callback = callback;
    }

    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        if self.marked_for_removal || self.path.is_empty() {
            self.mover.borrow_mut().sub_pos = (0.0, 0.0);
            return false;
        }

        let millis = util::get_elapsed_millis(self.start_time.elapsed());
        let frame_index = cmp::min((millis / self.frame_time_millis) as usize, self.path.len() - 1);
        let frame_frac = (millis % self.frame_time_millis) as f32 / self.frame_time_millis as f32;

        if frame_index != self.path.len() - 1 {
            let dx = self.smoothed_path[frame_index + 1].0 - self.smoothed_path[frame_index].0;
            let dy = self.smoothed_path[frame_index + 1].1 - self.smoothed_path[frame_index].1;

            let dx = dx * frame_frac;
            let dy = dy * frame_frac;

            let offset_x = self.smoothed_path[frame_index].0 - self.path[frame_index].x as f32;
            let offset_y = self.smoothed_path[frame_index].1 - self.path[frame_index].y as f32;

            self.mover.borrow_mut().sub_pos = (dx + offset_x, dy + offset_y);
        }

        if frame_index as i32 == self.last_frame_index {
            return true;
        }
        let move_ap = frame_index as i32 - self.last_frame_index;
        self.last_frame_index = frame_index as i32;

        let p = self.path[frame_index];
        let area_state = GameState::area_state();

        if !area_state.borrow_mut().move_entity(&self.mover, p.x, p.y, move_ap as u32) {
            self.mover.borrow_mut().sub_pos = (0.0, 0.0);
            return false;
        }

        trace!("Updated move animation at frame {}", frame_index);
        if frame_index == self.path.len() - 1 {
            self.mover.borrow_mut().sub_pos = (0.0, 0.0);

            if let Some(ref mut cb) = self.callback {
                cb.on_anim_complete();
            }

            return false;
        }


        true
    }

    fn is_blocking(&self) -> bool { true }

    fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
        self.mover.borrow_mut().sub_pos = (0.0, 0.0);
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.mover
    }
}
