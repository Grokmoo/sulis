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

use std::rc::Rc;
use std::option::Option;
use std::collections::HashMap;

use ui::{animation_state, AnimationState, Border, Callback, FontRenderer, Size};
use ui::theme::TextParams;

use util::Point;
use image::Image;
use resource::Font;

pub struct WidgetState {
    pub visible: bool,
    pub position: Point,
    pub size: Size,
    pub inner_size: Size,
    pub inner_position: Point,
    pub border: Border,
    pub mouse_is_inside: bool,
    pub background: Option<Rc<Image>>,
    pub foreground: Option<Rc<Image>>,
    pub animation_state: AnimationState,
    pub scroll_pos: Point,
    pub max_scroll_pos: Point,
    pub text: String,
    pub text_params: TextParams,
    pub text_renderer: Option<Box<FontRenderer>>,
    pub font: Option<Rc<Font>>,
    pub is_modal: bool,
    pub modal_remove_on_click_outside: bool,
    pub is_mouse_over: bool,

    text_args: HashMap<String, String>,
    pub (in ui) callback: Option<Callback>,
    pub (in ui) has_keyboard_focus: bool,
}

impl WidgetState {
    pub fn new() -> WidgetState {
        WidgetState {
            visible: true,
            callback: None,
            size: Size::as_zero(),
            position: Point::as_zero(),
            border: Border::as_zero(),
            mouse_is_inside: false,
            background: None,
            foreground: None,
            animation_state: AnimationState::default(),
            scroll_pos: Point::as_zero(),
            max_scroll_pos: Point::as_zero(),
            font: None,
            text: String::new(),
            text_params: TextParams::default(),
            text_renderer: None,
            text_args: HashMap::new(),
            inner_size: Size::as_zero(),
            inner_position: Point::as_zero(),
            is_modal: false,
            modal_remove_on_click_outside: false,
            is_mouse_over: false,
            has_keyboard_focus: false,
        }
    }

    pub fn has_keyboard_focus(&self) -> bool {
        self.has_keyboard_focus
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_active(&self) -> bool {
        self.animation_state.contains(animation_state::Kind::Active)
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.animation_state.add(animation_state::Kind::Active);
        } else {
            self.animation_state.remove(animation_state::Kind::Active);
        }
    }

    pub fn disable(&mut self) {
        self.animation_state.add(animation_state::Kind::Disabled);
        self.animation_state.remove(animation_state::Kind::Hover);
    }

    pub fn enable(&mut self) {
        self.animation_state.remove(animation_state::Kind::Disabled);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.enable();
        } else {
            self.disable();
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.animation_state.contains(animation_state::Kind::Disabled)
    }

    pub fn add_callback(&mut self, callback: Callback) {
        self.callback = Some(callback);
    }

    pub fn set_scroll(&mut self, x: i32, y: i32) -> bool {
        trace!("Setting scroll to {},{}", x, y);

        let x_result = self.set_scroll_x(x);
        let y_result = self.set_scroll_y(y);

        x_result && y_result
    }

    pub fn set_scroll_y(&mut self, y: i32) -> bool {
        if y < 0 || y >= self.max_scroll_pos.y - self.size.height + 1 {
            false
        } else {
            self.scroll_pos.set_y(y);
            true
        }
    }

    pub fn set_scroll_x(&mut self, x: i32) -> bool {
        if x < 0 || x >= self.max_scroll_pos.x - self.size.width + 1 {
            false
        } else {
            self.scroll_pos.set_x(x);
            true
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
        self.set_scroll_y(new_y)
    }

    pub fn scroll_x(&mut self, x: i32) -> bool {
        let new_x = self.scroll_pos.x + x;
        self.set_scroll_x(new_x)
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
    /// this is accessed by #id in the text format string.  Use
    /// '##' to produce one '#' character in the output
    pub fn add_text_arg(&mut self, id: &str, param: &str) {
        self.text_args.insert(id.to_string(), param.to_string());
    }

    /// clears all current text params, see `add_text_param`
    pub fn clear_text_args(&mut self) {
        self.text_args.clear();
    }

    pub fn has_text_arg(&self, id: &str) -> bool {
        self.text_args.contains_key(id)
    }

    pub fn get_text_arg(&self, id: &str) -> Option<&str> {
        self.text_args.get(id).map(|x| String::as_str(x))
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

    pub fn set_foreground(&mut self, image: Option<Rc<Image>>) {
        self.foreground = image;
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
