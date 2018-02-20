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
use {Button, Label};

const NAME: &str = "spinner";

pub struct Spinner {
    label: Rc<RefCell<Widget>>,
    down: Rc<RefCell<Widget>>,
    up: Rc<RefCell<Widget>>,
    value: i32,
    min: i32,
    max: i32,
}

impl Spinner {
    pub fn new(value: i32, min: i32, max: i32) -> Rc<RefCell<Spinner>> {
        Rc::new(RefCell::new(Spinner {
            label: Widget::with_defaults(Label::empty()),
            down: Widget::with_theme(Button::empty(), "down"),
            up: Widget::with_theme(Button::empty(), "up"),
            value,
            min,
            max,
        }))
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

impl WidgetKind for Spinner {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        self.label.borrow_mut().state.add_text_arg("value", &self.value.to_string());

        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.down.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            {
                let spinner = Widget::downcast_kind_mut::<Spinner>(&parent);
                if spinner.value > spinner.min {
                    spinner.value -= 1;
                }
            }
            parent.borrow_mut().invalidate_layout();
            let kind = Rc::clone(&parent.borrow().kind);
            let mut kind = kind.borrow_mut();
            Widget::fire_callback(&parent, &mut *kind);
        })));

        self.up.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            {
                let spinner = Widget::downcast_kind_mut::<Spinner>(&parent);
                if spinner.value < spinner.max {
                    spinner.value += 1;
                }
            }
            parent.borrow_mut().invalidate_layout();
            let kind = Rc::clone(&parent.borrow().kind);
            let mut kind = kind.borrow_mut();
            Widget::fire_callback(&parent, &mut *kind);
        })));

        vec![Rc::clone(&self.down), Rc::clone(&self.up), Rc::clone(&self.label)]
    }
}
