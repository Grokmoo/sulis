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

use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::util::{Point, Size};

use Scrollbar;

/// Simple ScrollPane that clips its child content to its inner area
/// nested scroll panes are not supported
/// only does vertical scrolling
pub struct ScrollPane {
    content: Rc<RefCell<Widget>>,
    scrollbar: Rc<RefCell<Scrollbar>>,
    scrollbar_widget: Rc<RefCell<Widget>>,
}

impl ScrollPane {
    pub fn new() -> Rc<RefCell<ScrollPane>> {
        let content = Widget::empty("content");
        let scrollbar = Scrollbar::new(&content);
        let scrollbar_widget = Widget::with_defaults(scrollbar.clone());
        Rc::new(RefCell::new(ScrollPane {
            content,
            scrollbar,
            scrollbar_widget,
        }))
    }

    pub fn add_to_content(&self, child: Rc<RefCell<Widget>>) {
        Widget::add_child_to(&self.content, child);
    }
}

impl WidgetKind for ScrollPane {
    widget_kind!["scrollpane"];

    fn draw(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
            widget: &Widget, _millis: u32) {
        let x = widget.state.inner_position.x;
        let y = widget.state.inner_position.y;
        let width = widget.state.inner_size.width;
        let height = widget.state.inner_size.height;
        renderer.set_scissor(Point::new(x, y), Size::new(width, height));
    }

    fn layout(&mut self, widget: &mut Widget) {
        let scroll = self.scrollbar.borrow().cur_y();

        widget.do_self_layout();
        let x = widget.state.position.x;
        let y = widget.state.position.y;
        widget.state.set_position(x, y - scroll);

        widget.do_children_layout();

        widget.state.set_position(x, y);

        let sx = self.scrollbar_widget.borrow().state.position.x;
        let sy = self.scrollbar_widget.borrow().state.position.y;
        self.scrollbar_widget.borrow_mut().state.set_position(sx, sy + scroll);
    }

    fn end_draw(&mut self, renderer: &mut GraphicsRenderer) {
        renderer.clear_scissor();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {

        vec![Rc::clone(&self.content), Rc::clone(&self.scrollbar_widget)]
    }
}
