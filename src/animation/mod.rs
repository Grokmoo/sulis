use std::time::Duration;
use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;

mod move_animation;
pub use self::move_animation::MoveAnimation;

use state::{AreaState, EntityState};

pub trait Animation {
    fn update(&self, area_state: &mut AreaState) -> bool;

    /// this method is called whenever another animation is added for the
    /// given entity.  This can be used to cancel this animation if the
    /// other animation should override it, for example.
    fn check(&mut self, entity: &Rc<RefCell<EntityState>>);
}

/// Helper function to return the number of milliseconds elapsed in
/// the given duration.
pub fn get_elapsed_millis(elapsed: Duration) -> u32 {
    (elapsed.as_secs() as u32) * 1_000 +
        elapsed.subsec_nanos() / 1_000_000
}

/// Helper function to return the number of frames elapsed for the 'elapsed'
/// duration.  This is equal to the duration divided by 'frame_time'.  The
/// value is capped by 'max_index'
pub fn get_current_frame(elapsed: Duration, frame_time:
                         u32, max_index: usize) -> usize {
    let millis = get_elapsed_millis(elapsed);

    let frame_index = (millis / frame_time) as usize;

    cmp::min(frame_index, max_index)
}
