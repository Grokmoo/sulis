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

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::image::Image;
use crate::io::{GraphicsRenderer, InputAction};
use crate::resource::ResourceSet;
use crate::ui::{Callback, LineRenderer, Widget, WidgetKind, RcRfc};
use crate::util::Point;
use crate::widgets::Label;

const NAME: &str = "input_field";

type KeyPressCallback = Rc<dyn Fn(&Rc<RefCell<Widget>>, &mut InputField, InputAction)>;

pub struct InputField {
    pub text: String,
    label: RcRfc<Label>,
    carat: Option<Rc<dyn Image>>,
    carat_width: f32,
    carat_height: f32,
    carat_offset: f32,
    enter_callback: Option<Callback>,
    key_press_callback: Option<KeyPressCallback>,
    ignore_next: bool, // hack to prevent console from receiving a character
}

impl InputField {
    pub fn new(text: &str) -> RcRfc<InputField> {
        Rc::new(RefCell::new(InputField {
            text: text.to_string(),
            label: Label::new(text),
            carat: None,
            carat_width: 1.0,
            carat_height: 1.0,
            carat_offset: 0.0,
            enter_callback: None,
            key_press_callback: None,
            ignore_next: false,
        }))
    }

    pub fn set_ignore_next(&mut self) {
        self.ignore_next = true;
    }

    pub fn set_key_press_callback(&mut self, cb: KeyPressCallback) {
        self.key_press_callback = Some(cb);
    }

    pub fn set_enter_callback(&mut self, cb: Callback) {
        self.enter_callback = Some(cb);
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn set_text(&mut self, text: &str, widget: &RcRfc<Widget>) {
        self.text = text.to_string();
        self.label.borrow_mut().text = Some(self.text.clone());
        widget.borrow_mut().invalidate_layout();
    }

    pub fn clear(&mut self, widget: &RcRfc<Widget>) {
        self.text.clear();
        self.label.borrow_mut().text = Some(self.text.clone());
        widget.borrow_mut().invalidate_layout();
    }
}

impl WidgetKind for InputField {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn wants_keyboard_focus(&self) -> bool {
        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }

        let theme = &widget.theme;
        if let Some(ref image_id) = theme.custom.get("carat_image") {
            self.carat = ResourceSet::image(image_id);
        }

        self.carat_width = theme.get_custom_or_default("carat_width", 1.0);
        self.carat_height = theme.get_custom_or_default("carat_height", 1.0);
        self.carat_offset = theme.get_custom_or_default("carat_offset", 0.0);
    }

    fn on_char_typed(&mut self, widget: &RcRfc<Widget>, c: char) -> bool {
        if self.ignore_next {
            self.ignore_next = false;
            return true;
        }

        match c {
            '\u{8}' => {
                self.text.pop();
            }
            _ => {
                if self.label.borrow().text_draw_end_x > widget.borrow().state.inner_right() as f32
                {
                    return true;
                }

                self.text.push(c);
            }
        }

        let mut buf = [0u8; 4];
        c.encode_utf8(&mut buf);
        if buf[0] == 13 && c.len_utf8() == 1 {
            let cb = self.enter_callback.clone();
            if let Some(cb) = cb {
                (cb).call(widget, self);
            }
        }

        self.label.borrow_mut().text = Some(self.text.clone());
        widget.borrow_mut().invalidate_layout();
        Widget::fire_callback(widget, self);
        true
    }

    fn on_key_press(&mut self, widget: &RcRfc<Widget>, key: InputAction) -> bool {
        let cb = self.key_press_callback.clone();
        if let Some(ref cb) = cb {
            (cb)(widget, self, key);
        }

        true
    }

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
        self.label
            .borrow_mut()
            .draw(renderer, pixel_size, widget, millis);

        if !widget.state.has_keyboard_focus() {
            return;
        }

        if let Some(ref carat) = self.carat {
            let x = self.label.borrow().text_draw_end_x + self.carat_offset;
            let y = widget.state.inner_top() as f32
                + (widget.state.inner_height() as f32 - self.carat_height) / 2.0;
            let w = self.carat_width;
            let h = self.carat_height;
            carat.draw(renderer, &widget.state.animation_state, x, y, w, h, millis);
        }
    }
}
