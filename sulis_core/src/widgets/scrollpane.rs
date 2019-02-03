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

use crate::widget_kind;
use crate::io::{InputAction, GraphicsRenderer};
use crate::ui::{Callback, Widget, WidgetKind};
use crate::util::{Point, Size};

use crate::widgets::{Button, TextArea};

/// Simple ScrollPane that clips its child content to its inner area
/// nested scroll panes are not supported
/// only does vertical scrolling
pub struct ScrollPane {
    content: Rc<RefCell<Widget>>,
    content_height: i32,
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
            content_height: 0,
            scrollbar,
            scrollbar_widget,
        }))
    }

    pub fn add_to_content(&self, child: Rc<RefCell<Widget>>) {
        // for text areas inside scroll panes don't limit drawing
        // inside the screen area on layout as scroll panes
        // need to allow widgets to be off screen
        disable_text_area_limit_recursive(&child);

        Widget::add_child_to(&self.content, child);
    }
}

fn disable_text_area_limit_recursive(parent: &Rc<RefCell<Widget>>) {
    let kind = Rc::clone(&parent.borrow().kind);
    match kind.borrow_mut().as_any_mut().downcast_mut::<TextArea>() {
        None => (),
        Some(text_area) => {
            text_area.limit_to_screen_edge = false;
        }
    }

    for child in parent.borrow().children.iter() {
        disable_text_area_limit_recursive(child);
    }
}

impl WidgetKind for ScrollPane {
    widget_kind!["scrollpane"];

    fn draw(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
            widget: &Widget, _millis: u32) {
        let x = widget.state.inner_left();
        let y = widget.state.inner_top();
        let width = widget.state.inner_width();
        let height = widget.state.inner_height();
        renderer.set_scissor(Point::new(x, y), Size::new(width, height));
    }

    fn on_key_press(&mut self, _widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        let delta = match key {
            InputAction::ZoomIn => -1,
            InputAction::ZoomOut => 1,
            _ => return false,
        };

        self.scrollbar.borrow_mut().update_children_position(&self.content, delta);

        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        let scroll = self.scrollbar.borrow().cur_y();

        widget.do_self_layout();
        let (x, y) = widget.state.position().as_tuple();
        widget.state.set_position(x, y - scroll);

        widget.do_children_layout();

        widget.state.set_position(x, y);

        let (sx, sy) = self.scrollbar_widget.borrow().state.position().as_tuple();
        self.scrollbar_widget.borrow_mut().state.set_position(sx, sy + scroll);

        if self.content_height == 0 {
            self.content_height = self.content.borrow().state.height();
            self.scrollbar.borrow_mut().content_height = self.content_height;
        }

        let w = self.content.borrow().state.width();
        self.content.borrow_mut().state.set_size(Size::new(w, self.content_height + scroll));
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
    content_height: i32,
    delta: u32,

    min_y: i32,
    cur_y: i32,
    max_y: i32,

    up: Rc<RefCell<Widget>>,
    down: Rc<RefCell<Widget>>,
    thumb: Rc<RefCell<Widget>>,
}

impl Scrollbar {
    fn new(widget_to_scroll: &Rc<RefCell<Widget>>) -> Rc<RefCell<Scrollbar>> {
        let up = Widget::with_theme(Button::empty(), "up");
        let down = Widget::with_theme(Button::empty(), "down");
        let thumb = Widget::with_theme(Button::empty(), "thumb");

        Rc::new(RefCell::new(Scrollbar {
            widget: Rc::clone(widget_to_scroll),
            delta: 1,
            cur_y: 0,
            min_y: 0,
            max_y: 0,
            up,
            down,
            thumb,
            content_height: 0,
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

        let pane = Widget::direct_parent(&self.widget);
        pane.borrow_mut().invalidate_layout();
    }

    fn compute_min_max_y(&mut self, widget: &Rc<RefCell<Widget>>) {
        let mut max_y = 0;

        let len = widget.borrow().children.len();
        for i in 0..len {
            let child = Rc::clone(&widget.borrow().children[i]);

            max_y = cmp::max(max_y, child.borrow().state.top() + child.borrow().state.height());
        }

        self.max_y = max_y - self.content_height + widget.borrow().state.border().vertical()
            - widget.borrow().state.top();

        if self.max_y < self.min_y { self.max_y = self.min_y }
    }
}

impl WidgetKind for Scrollbar {
    widget_kind!("scrollbar");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if self.content_height == 0 {
            self.content_height = self.widget.borrow().state.height();
        }

        let widget_ref = Rc::clone(&self.widget);
        self.compute_min_max_y(&widget_ref);

        self.delta = widget.theme.get_custom_or_default("scroll_delta", 1);

        self.up.borrow_mut().state.set_enabled(self.cur_y != self.min_y);
        self.down.borrow_mut().state.set_enabled(self.cur_y != self.max_y);


        let widget_height = self.content_height as f32;
        let thumb_frac = widget_height / ((self.max_y as f32 - self.min_y as f32) + widget_height);

        self.thumb.borrow_mut().state.set_enabled(false);
        widget.state.set_visible(thumb_frac < 1.0);

        let inner_height = widget.state.inner_height() - self.up.borrow().state.height() -
            self.down.borrow().state.height();
        let thumb_height = (thumb_frac * inner_height as f32).round() as i32;
        let thumb_width = self.thumb.borrow().state.width();
        self.thumb.borrow_mut().state.set_size(Size::new(thumb_width, thumb_height));

        let thumb_x = self.thumb.borrow().state.left();
        let thumb_base_y = self.up.borrow().state.top() + self.up.borrow().state.height();

        let thumb_pos_frac = (self.cur_y as f32 - self.min_y as f32) / (self.max_y as f32 - self.min_y as f32);
        let thumb_max_y = self.down.borrow().state.top() - thumb_height - thumb_base_y;

        let thumb_y = thumb_base_y + (thumb_pos_frac * thumb_max_y as f32).round() as i32;

        self.thumb.borrow_mut().state.set_position(thumb_x, thumb_y);
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let widget_ref = Rc::clone(&self.widget);
        self.up.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let (_, kind) = Widget::parent_mut::<Scrollbar>(&widget);
            kind.update_children_position(&widget_ref, -1);
        })));

        let widget_ref = Rc::clone(&self.widget);
        self.down.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let (_, kind) = Widget::parent_mut::<Scrollbar>(&widget);
            kind.update_children_position(&widget_ref, 1);
        })));

        vec![Rc::clone(&self.up), Rc::clone(&self.down), Rc::clone(&self.thumb)]
    }
}
