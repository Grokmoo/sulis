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
use std::cmp;

use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::{Point, Size};

use Button;

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

struct Scrollbar {
    widget: Rc<RefCell<Widget>>,
    delta: u32,

    min_y: i32,
    cur_y: i32,
    max_y: i32,

    up: Rc<RefCell<Widget>>,
    down: Rc<RefCell<Widget>>,
}

impl Scrollbar {
    fn new(widget_to_scroll: &Rc<RefCell<Widget>>) -> Rc<RefCell<Scrollbar>> {
        let up = Widget::with_theme(Button::empty(), "up");
        let down = Widget::with_theme(Button::empty(), "down");

        Rc::new(RefCell::new(Scrollbar {
            widget: Rc::clone(widget_to_scroll),
            delta: 1,
            cur_y: 0,
            min_y: 0,
            max_y: 0,
            up,
            down,
        }))
    }

    fn cur_y(&self) -> i32 {
        self.cur_y
    }

    fn update_children_position(&mut self, parent: &Rc<RefCell<Widget>>, dir: i32) {
        self.compute_min_max_y(parent);

        self.cur_y += dir * self.delta as i32;

        if self.cur_y < self.min_y { self.cur_y = self.min_y }
        else if self.cur_y > self.max_y { self.cur_y = self.max_y }

        let pane = Widget::get_parent(&self.widget);
        pane.borrow_mut().invalidate_layout();
    }

    fn compute_min_max_y(&mut self, widget: &Rc<RefCell<Widget>>) {
        let mut max_y = 0;

        let len = widget.borrow().children.len();
        for i in 0..len {
            let child = Rc::clone(&widget.borrow().children[i]);

            max_y = cmp::max(max_y, child.borrow().state.position.y + child.borrow().state.size.height);
        }

        self.max_y = max_y - widget.borrow().state.inner_size.height - widget.borrow().state.position.y;

        if self.max_y < self.min_y { self.max_y = self.min_y }
    }
}

impl WidgetKind for Scrollbar {
    widget_kind!("scrollbar");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        let widget_ref = Rc::clone(&self.widget);
        self.compute_min_max_y(&widget_ref);

        if let Some(ref theme) = widget.theme {
            self.delta = theme.get_custom_or_default("scroll_delta", 1);
        }

        self.up.borrow_mut().state.set_enabled(self.cur_y != self.min_y);
        self.down.borrow_mut().state.set_enabled(self.cur_y != self.max_y);
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let widget_ref = Rc::clone(&self.widget);
        self.up.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            let kind = Widget::downcast_kind_mut::<Scrollbar>(&parent);
            kind.update_children_position(&widget_ref, -1);
        })));

        let widget_ref = Rc::clone(&self.widget);
        self.down.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            let kind = Widget::downcast_kind_mut::<Scrollbar>(&parent);
            kind.update_children_position(&widget_ref, 1);
        })));

        vec![Rc::clone(&self.up), Rc::clone(&self.down)]
    }
}
