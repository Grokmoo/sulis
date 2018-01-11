use std::rc::Rc;
use std::option::Option;

use ui::{Border, Size, AnimationState};
use ui::theme::TextParams;

use util::Point;
use image::Image;
use resource::Font;

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
    pub text_params: TextParams,
    pub font: Option<Rc<Font>>,
    pub is_modal: bool,
    pub is_mouse_over: bool,

    pub text_args: Vec<String>,
}

impl WidgetState {
    pub fn new() -> WidgetState {
        WidgetState {
            size: Size::as_zero(),
            position: Point::as_zero(),
            border: Border::as_zero(),
            mouse_is_inside: false,
            background: None,
            animation_state: AnimationState::default(),
            scroll_pos: Point::as_zero(),
            max_scroll_pos: Point::as_zero(),
            font: None,
            text: String::new(),
            text_params: TextParams::default(),
            text_args: Vec::new(),
            inner_size: Size::as_zero(),
            inner_position: Point::as_zero(),
            is_modal: false,
            is_mouse_over: false,
        }
    }

    pub fn scroll(&mut self, x: i32, y: i32) -> bool {
        trace!("Scrolling by {},{}", x, y);
        // make sure both components are attempted independently
        let x_bool = self.scroll_x(x);
        let y_bool = self.scroll_y(y);

        x_bool && y_bool
    }

    pub fn scroll_y(&mut self, y: i32) -> bool {
        let new_y = self.scroll_pos.y + y;

        if new_y < 0 || new_y >= self.max_scroll_pos.y - self.size.height + 1 {
            false
        } else {
            self.scroll_pos.set_y(new_y);
            true
        }
    }

    pub fn scroll_x(&mut self, x: i32) -> bool {
        let new_x = self.scroll_pos.x + x;

        if new_x < 0 || new_x >= self.max_scroll_pos.x - self.size.width + 1 {
            false
        } else {
            self.scroll_pos.set_x(new_x);
            true
        }
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

    pub fn inner_top(&self) -> i32 {
        self.position.y + self.border.top
    }

    pub fn inner_left(&self) -> i32 {
        self.position.x + self.border.left
    }

    pub fn set_max_scroll_pos(&mut self, x: i32, y: i32) {
        self.max_scroll_pos.set(x, y);
        self.scroll_pos.min(x, y);
    }

    /// Adds a text argument to the list of text args stored in this
    /// state.  When building the output text for the owning widget,
    /// this is accessed by #n in the text format string, where n is
    /// an integer 0 to 9.  Use '##' to produce one '#' character in
    /// the output
    pub fn add_text_arg(&mut self, param: &str) {
        self.text_args.push(param.to_string());
    }

    /// clears all current text params, see `add_text_param`
    pub fn clear_text_args(&mut self) {
        self.text_args.clear();
    }

    pub fn get_text_arg(&self, index: u32) -> Option<&str> {
        self.text_args.get(index as usize).map(|x| String::as_str(x))
    }

    pub fn set_text_content(&mut self, text: String) {
        self.text = text;
    }

    pub fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    pub fn set_animation_state(&mut self, state: &AnimationState) {
        self.animation_state = state.clone();
    }

    pub fn set_background(&mut self, image: Option<Rc<Image>>) {
        self.background = image;
    }

    pub(super) fn set_mouse_inside(&mut self, is_inside: bool) {
        self.mouse_is_inside = is_inside;
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.size.in_bounds(x - self.position.x, y - self.position.y)
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
