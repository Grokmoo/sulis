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

use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::io::{GraphicsRenderer};
use sulis_core::util::Point;
use sulis_widgets::{TextArea};

const NAME: &'static str = "prop_mouseover";

pub struct BasicMouseover {
    text_area: Rc<RefCell<TextArea>>,
    text: String,
}

impl BasicMouseover {
    pub fn new(text: &str) -> Rc<RefCell<BasicMouseover>> {
        Rc::new(RefCell::new(BasicMouseover {
            text_area: TextArea::empty(),
            text: text.to_string(),
        }))
    }
}

impl WidgetKind for BasicMouseover {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.state.clear_text_args();
        widget.state.add_text_arg("name", &self.text);

        self.text_area.borrow_mut().layout(widget);
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
            widget: &Widget, millis: u32) {
        self.text_area.borrow_mut().draw(renderer, pixel_size, widget, millis);
    }
}
