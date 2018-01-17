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
