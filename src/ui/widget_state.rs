use std::rc::Rc;

use ui::{Border, Size, AnimationState};

use resource::Point;
use resource::Image;

//// The base widget holder class.  Contains the common implementation across all
//// widgets, and holds an instance of 'Widget' which contains the specific behavior.
pub struct WidgetState {
    pub position: Point,
    pub size: Size,
    pub border: Border,
    pub mouse_is_inside: bool,
    pub background: Option<Rc<Image>>,
    pub animation_state: AnimationState,
}

impl WidgetState {
    pub fn new(size: Size, position: Point,
               border: Border) -> WidgetState {

        WidgetState {
            size,
            position,
            border,
            mouse_is_inside: false,
            background: None,
            animation_state: AnimationState::Base,
        }
    }

    pub fn set_animation_state(&mut self, state: AnimationState) {
        self.animation_state = state;
    }

    pub fn set_background(&mut self, image: Option<Rc<Image>>) {
        self.background = image;
    }

    pub(super) fn set_mouse_inside(&mut self, is_inside: bool) {
        self.mouse_is_inside = is_inside;
    }

    pub fn inner_position(&self) -> Point {
        self.position.inner(&self.border)
    }

    pub fn inner_size(&self) -> Size {
        self.size.inner(&self.border)
    }

    pub fn in_bounds(&self, p: Point) -> bool {
        self.size.in_bounds(p.x - self.position.x as i32,
                            p.y - self.position.y as i32)
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.position = Point::new(
            x - (self.size.width - 1) / 2,
            y - (self.size.height - 1) / 2);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }
}
