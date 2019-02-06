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

use std::collections::HashMap;
use std::option::Option;
use std::rc::Rc;

use crate::ui::theme::TextParams;
use crate::ui::{animation_state, AnimationState, Border, Callback, FontRenderer, Size};

use crate::image::Image;
use crate::resource::Font;
use crate::util::Point;

pub struct WidgetState {
    pub visible: bool,
    position: Point,
    size: Size,
    border: Border,
    pub mouse_is_inside: bool,
    pub background: Option<Rc<Image>>,
    pub foreground: Option<Rc<Image>>,
    pub animation_state: AnimationState,
    pub text: String,
    pub text_params: TextParams,
    pub text_renderer: Option<Box<FontRenderer>>,
    pub font: Option<Rc<Font>>,
    pub is_modal: bool,
    pub modal_remove_on_click_outside: bool,
    pub is_mouse_over: bool,
    text_args: HashMap<String, String>,
    pub(in crate::ui) callback: Option<Callback>,
    pub(in crate::ui) has_keyboard_focus: bool,
}

impl WidgetState {
    pub fn new() -> WidgetState {
        WidgetState {
            visible: true,
            callback: None,
            size: Size::default(),
            position: Point::default(),
            border: Border::default(),
            mouse_is_inside: false,
            background: None,
            foreground: None,
            animation_state: AnimationState::default(),
            font: None,
            text: String::new(),
            text_params: TextParams::default(),
            text_renderer: None,
            text_args: HashMap::new(),
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
        !self
            .animation_state
            .contains(animation_state::Kind::Disabled)
    }

    pub fn add_callback(&mut self, callback: Callback) {
        self.callback = Some(callback);
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.is_modal = modal;
    }

    pub fn border(&self) -> Border {
        self.border
    }
    pub fn position(&self) -> Point {
        self.position
    }
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn width(&self) -> i32 {
        self.size.width
    }
    pub fn height(&self) -> i32 {
        self.size.height
    }

    pub fn inner_position(&self) -> Point {
        Point::new(self.inner_left(), self.inner_top())
    }

    pub fn inner_size(&self) -> Size {
        Size::new(self.inner_width(), self.inner_height())
    }

    pub fn left(&self) -> i32 {
        self.position.x
    }
    pub fn top(&self) -> i32 {
        self.position.y
    }
    pub fn right(&self) -> i32 {
        self.position.x + self.size.width
    }
    pub fn bottom(&self) -> i32 {
        self.position.y + self.size.height
    }

    pub fn inner_width(&self) -> i32 {
        self.size.width - self.border.horizontal()
    }
    pub fn inner_height(&self) -> i32 {
        self.size.height - self.border.vertical()
    }

    pub fn inner_left(&self) -> i32 {
        self.position.x + self.border.left
    }
    pub fn inner_top(&self) -> i32 {
        self.position.y + self.border.top
    }
    pub fn inner_right(&self) -> i32 {
        self.right() - self.border.right
    }
    pub fn inner_bottom(&self) -> i32 {
        self.bottom() - self.border.bottom
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
        self.size
            .in_bounds(x - self.position.x, y - self.position.y)
    }

    pub fn set_border(&mut self, border: Border) {
        self.border = border;
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.position = Point::new(
            x - (self.size.width - 1) / 2,
            y - (self.size.height - 1) / 2,
        );
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }

    //// Returns a point which will cause a widget with the specified
    //// size to be centered on this widget.  This is useful when
    //// centering a widget on a larger parent widget.
    pub fn get_centered_position(&self, width: i32, height: i32) -> Point {
        Point::new(
            self.inner_left() + (self.size.width - width) / 2,
            self.inner_top() + (self.size.height - height) / 2,
        )
    }
}
