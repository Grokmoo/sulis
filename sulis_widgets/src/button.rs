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
use sulis_core::io::{event, GraphicsRenderer};
use sulis_core::util::Point;
use Label;

pub struct Button {
    label: Rc<RefCell<Label>>,
}

impl Button {
    pub fn empty() -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label: Label::empty(),
        }))
    }

    pub fn with_text(text: &str) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label: Label::new(text),
        }))
    }
}

impl WidgetKind for Button {
    widget_kind!["button"];

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.label.borrow_mut().draw_graphics_mode(renderer, pixel_size, widget, millis);
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        Widget::fire_callback(widget, self);
        true
    }
}
