use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use state::{AreaState, EntityState};
use grt::util::Point;
use animation;

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
        if self.marked_for_removal {
            return false;
        }

        let frame_index = animation::get_current_frame(self.start_time.elapsed(),
            self.frame_time_millis, self.path.len() - 1);

        if frame_index as i32 == self.last_frame_index {
            return true;
        }
        let move_ap = frame_index as i32 - self.last_frame_index;
        self.last_frame_index = frame_index as i32;

        let p = self.path[frame_index];
        if !&self.mover.borrow_mut().move_to(area_state, p.x, p.y, move_ap as u32) {
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
