use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use state::EntityState;
use resource::Point;
use animation;

pub struct MoveAnimation<'a> {
   mover: Rc<RefCell<EntityState<'a>>>,
   path: Vec<Point>,
   start_time: Instant,
   frame_time_millis: u32,
}

impl<'a> MoveAnimation<'a> {
    pub fn new(mover: Rc<RefCell<EntityState<'a>>>,
               path: Vec<Point>, frame_time_millis: u32) -> MoveAnimation<'a> {

        MoveAnimation {
            mover,
            path,
            start_time: Instant::now(),
            frame_time_millis,
        }
    }

}

impl<'a> animation::Animation for MoveAnimation<'a> {
    fn update(&self) -> bool {
        let frame_index = animation::get_current_frame(self.start_time.elapsed(),
            self.frame_time_millis, self.path.len() - 1);

        let p = self.path.get(frame_index).unwrap();
        &self.mover.borrow_mut().move_to(p.x, p.y);

        trace!("Updated move animation at frame {}", frame_index);
        return frame_index != self.path.len() - 1
    }
}
