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

use crate::ui::{Callback, Widget, WidgetKind};
use crate::widgets::{Button, Label};

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

    pub fn set_max(&mut self, max: i32) {
        self.max = max;
    }

    pub fn set_min(&mut self, min: i32) {
        self.min = min;
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
        if value > self.max {
            self.value = self.max;
        } else if value < self.min {
            self.value = self.min;
        }
    }
}

impl WidgetKind for Spinner {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn layout(&mut self, widget: &mut Widget) {
        self.label
            .borrow_mut()
            .state
            .add_text_arg("value", &self.value.to_string());

        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.down
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, spinner) = Widget::parent_mut::<Spinner>(widget);
                if spinner.value > spinner.min {
                    spinner.value -= 1;
                }

                parent.borrow_mut().invalidate_layout();
                Widget::fire_callback(&parent, &mut *spinner);
            })));

        self.up
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, spinner) = Widget::parent_mut::<Spinner>(widget);
                if spinner.value < spinner.max {
                    spinner.value += 1;
                }
                parent.borrow_mut().invalidate_layout();
                Widget::fire_callback(&parent, &mut *spinner);
            })));

        vec![
            Rc::clone(&self.down),
            Rc::clone(&self.up),
            Rc::clone(&self.label),
        ]
    }
}
