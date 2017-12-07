use std::time::Duration;
use std::cmp;

mod move_animation;
pub use self::move_animation::MoveAnimation;

pub trait Animation {
    fn update(&self) -> bool;
}

//// Helper function to return the number of milliseconds elapsed in
//// the given duration.
pub fn get_elapsed_millis(elapsed: Duration) -> u32 {
    (elapsed.as_secs() as u32) * 1_000 +
        elapsed.subsec_nanos() / 1_000_000
}

//// Helper function to return the number of frames elapsed for the 'elapsed'
//// duration.  This is equal to the duration divided by 'frame_time'.  The
//// value is capped by 'max_index'
pub fn get_current_frame(elapsed: Duration, frame_time:
                         u32, max_index: usize) -> usize {
    let millis = get_elapsed_millis(elapsed);

    let frame_index = (millis / frame_time) as usize;

    cmp::min(frame_index, max_index)
}
