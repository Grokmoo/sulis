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
use sulis_state::{GameState};

const NAME: &'static str = "prop_mouseover";

pub struct PropMouseover {
    prop_index: usize,
    text_area: Rc<RefCell<TextArea>>,
}

impl PropMouseover {
    pub fn new(prop_index: usize) -> Rc<RefCell<PropMouseover>> {
        Rc::new(RefCell::new(PropMouseover {
            prop_index,
            text_area: TextArea::empty(),
        }))
    }
}

impl WidgetKind for PropMouseover {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let prop = area_state.get_prop(self.prop_index);

        widget.state.clear_text_args();
        if !prop.might_contain_items() {
            widget.state.add_text_arg("empty", "true");
        }
        widget.state.add_text_arg("name", &prop.prop.name);

        // double layout - first to get the position, then to actually do the layout
        self.text_area.borrow_mut().layout(widget);
        widget.state.position.y -= widget.state.size.height;
        widget.state.position.x -= widget.state.size.width / 2;
        self.text_area.borrow_mut().layout(widget);
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.text_area.borrow_mut().draw_graphics_mode(renderer, pixel_size, widget, millis);
    }
}
