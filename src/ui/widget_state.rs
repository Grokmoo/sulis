use std::rc::Rc;

use ui::{Border, Size, AnimationState};

use resource::Point;
use resource::Image;

//// The base widget holder class.  Contains the common implementation across all
//// widgets, and holds an instance of 'Widget' which contains the specific behavior.
pub struct WidgetState {
    pub position: Point,
    pub size: Size,
    pub inner_size: Size,
    pub inner_position: Point,
    pub border: Border,
    pub mouse_is_inside: bool,
    pub background: Option<Rc<Image>>,
    pub animation_state: AnimationState,
    pub scroll_pos: Point,
    pub max_scroll_pos: Point,
    pub text: String,
    pub is_modal: bool,
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
            scroll_pos: Point::as_zero(),
            max_scroll_pos: Point::as_zero(),
            text: String::new(),
            inner_size: size.inner(&border),
            inner_position: position.inner(&border),
            is_modal: false,
        }
    }

    pub fn scroll(&mut self, x: i32, y: i32) -> bool {
        trace!("Scrolling by {},{}", x, y);
        let new_x = self.scroll_pos.x + x;
        let new_y = self.scroll_pos.y + y;

        if new_x < 0 || new_y < 0 { return false; }

        if new_x >= self.max_scroll_pos.x - self.size.width + 1 ||
            new_y >= self.max_scroll_pos.y - self.size.height + 1 {
            return false;
        }

        self.scroll_pos.set(new_x, new_y);

        true
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.is_modal = modal;
    }

    pub fn inner_right(&self) -> i32 {
        self.position.x + self.size.width - self.border.right
    }

    pub fn inner_bottom(&self) -> i32 {
        self.position.y + self.size.height - self.border.bottom
    }

    pub fn inner_top(&self) -> i32{
        self.position.y + self.border.top
    }

    pub fn inner_left(&self) -> i32 {
        self.position.x + self.border.left
    }

    pub fn set_max_scroll_pos(&mut self, x: i32, y: i32) {
        self.max_scroll_pos.set(x, y);
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
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

    pub fn in_bounds(&self, p: Point) -> bool {
        self.size.in_bounds(p.x - self.position.x as i32,
                            p.y - self.position.y as i32)
    }

    pub fn set_border(&mut self, border: Border) {
        self.border = border;
        self.inner_size = self.size.inner(&border);
        self.inner_position = self.position.inner(&border);
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
        self.inner_size = self.size.inner(&self.border);
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.position = Point::new(
            x - (self.size.width - 1) / 2,
            y - (self.size.height - 1) / 2);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
        self.inner_position = self.position.inner(&self.border);
    }

    //// Returns a point which will cause a widget with the specified
    //// size to be centered on this widget.  This is useful when
    //// centering a widget on a larger parent widget.
    pub fn get_centered_position(&self, width: i32, height: i32) -> Point {
        Point::new(self.inner_left() + (self.size.width - width) / 2,
            self.inner_top() + (self.size.height - height) / 2)
    }
}
