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
use std::fmt::Display;
use std::slice::Iter;

use list_box::Entry;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use {Button, ListBox};

const NAME: &str = "drop_down";

pub struct DropDown<T: Display + Clone + 'static> {
    entries: Vec<Entry<T>>,
}

impl<T: Display + Clone + 'static> DropDown<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<RefCell<DropDown<T>>> {
        Rc::new(RefCell::new(DropDown {
            entries,
        }))
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries.get(index)
    }
}

impl<T: Display + Clone + 'static> WidgetKind for DropDown<T> {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let button = Widget::with_defaults(Button::empty());
        let entries_clone = self.entries.clone();
        let cb = Callback::new(Rc::new(move |widget, _kind| {
            let list_box = Widget::with_theme(ListBox::new(entries_clone.clone()), "list");
            list_box.borrow_mut().state.set_modal(true);
            list_box.borrow_mut().state.modal_remove_on_click_outside = true;
            let parent = Widget::get_root(widget);
            Widget::add_child_to(&parent, list_box);
        }));
        button.borrow_mut().state.add_callback(cb);

        vec![button]
    }
}
