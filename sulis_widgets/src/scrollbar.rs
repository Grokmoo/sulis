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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use {Button};

const NAME: &str = "scrollbar";

pub struct Scrollbar {
    widget: Rc<RefCell<Widget>>,
    delta: u32,
}

impl Scrollbar {
    pub fn new(widget_to_scroll: &Rc<RefCell<Widget>>) -> Rc<RefCell<Scrollbar>> {
        Rc::new(RefCell::new(Scrollbar {
            widget: Rc::clone(widget_to_scroll),
            delta: 1,
        }))
    }
}

impl WidgetKind for Scrollbar {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if let Some(ref theme) = widget.theme {
            self.delta = theme.get_custom_or_default("scroll_delta", 1);
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let up = Widget::with_theme(Button::empty(), "up");

        let widget_ref = Rc::clone(&self.widget);
        up.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            let kind = Widget::downcast_kind::<Scrollbar>(&parent);
            update_children_position(&widget_ref, 0, kind.delta as i32);
        })));

        let down = Widget::with_theme(Button::empty(), "down");

        let widget_ref = Rc::clone(&self.widget);
        down.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            let kind = Widget::downcast_kind::<Scrollbar>(&parent);
            update_children_position(&widget_ref, 0, -(kind.delta as i32));
        })));

        vec![up, down]
    }
}

fn update_children_position(parent: &Rc<RefCell<Widget>>, delta_x: i32, delta_y: i32) {
    let len = parent.borrow().children.len();
    for i in 0..len {
        let child = Rc::clone(&parent.borrow().children[i]);
        let cur_x = child.borrow().state.position.x;
        let cur_y = child.borrow().state.position.y;
        child.borrow_mut().state.set_position(cur_x + delta_x, cur_y + delta_y);

        update_children_position(&child, delta_x, delta_y);
    }
}
