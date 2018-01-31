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
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{LineRenderer, Widget, WidgetKind};
use sulis_core::io::{GraphicsRenderer};
use sulis_core::util::Point;
use Label;

const NAME: &str = "input_field";

pub struct InputField {
    pub text: String,
    label: Rc<RefCell<Label>>,
}

impl InputField {
    pub fn new() -> Rc<RefCell<InputField>> {
        Rc::new(RefCell::new(InputField {
            text: String::new(),
            label: Label::empty(),
        }))
    }
}

impl WidgetKind for InputField {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn wants_keyboard_focus(&self) -> bool { true }

    fn layout(&self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }
    }

    fn on_char_typed(&mut self, widget: &Rc<RefCell<Widget>>, c: char) -> bool {
        match c {
            '\u{8}' => { self.text.pop(); },
            _ => self.text.push(c),
        }

        self.label.borrow_mut().text = Some(self.text.clone());
        widget.borrow_mut().invalidate_layout();
        true
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.label.borrow_mut().draw_graphics_mode(renderer, pixel_size, widget, millis);
    }
}
