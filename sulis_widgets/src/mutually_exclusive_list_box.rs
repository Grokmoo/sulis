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
use std::fmt::Display;
use std::rc::Rc;
use std::slice::Iter;
use std::cell::RefCell;

use sulis_core::ui::{Callback, Widget, WidgetKind};

use {ListBox};
use list_box::{self, Entry};

pub struct MutuallyExclusiveListBox<T: Display + Clone + 'static> {
    list_box: ListBox<T>,
    active_entry: Option<Entry<T>>,
    cb: Rc<Fn(Option<&Entry<T>>)>,
}

impl<T: Display + Clone + 'static> MutuallyExclusiveListBox<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<RefCell<MutuallyExclusiveListBox<T>>> {
        Rc::new(RefCell::new(MutuallyExclusiveListBox {
            list_box: ListBox {
                entries,
            },
            cb: Rc::new(|_entry| {}),
            active_entry: None,
        }))
    }

    pub fn with_callback(entries: Vec<Entry<T>>, cb: Rc<Fn(Option<&Entry<T>>)>)
        -> Rc<RefCell<MutuallyExclusiveListBox<T>>> {
        Rc::new(RefCell::new(MutuallyExclusiveListBox {
            list_box: ListBox { entries },
            cb,
            active_entry: None,
        }))
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.list_box.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.list_box.get(index)
    }

    pub fn active_entry(&self) -> Option<&Entry<T>> {
        self.active_entry.as_ref()
    }

    pub fn has_active_entry(&self) -> bool {
        self.active_entry.is_some()
    }
}

impl<T: Display + Clone> WidgetKind for MutuallyExclusiveListBox<T> {
    fn get_name(&self) -> &str { list_box::NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children = Vec::new();
        for widget in self.list_box.on_add(widget) {
            widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _kind| {
                let parent = Widget::get_parent(widget);
                let parent_list_box = Widget::downcast_kind_mut::<MutuallyExclusiveListBox<T>>(&parent);

                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    for (index, child) in parent.borrow().children.iter().enumerate() {
                        if Rc::ptr_eq(child, widget) {
                            parent_list_box.active_entry = match parent_list_box.get(index) {
                                None => None,
                                Some(entry) => Some(entry.clone()),
                            };
                        }
                        child.borrow_mut().state.set_active(false);
                    }
                } else {
                    parent_list_box.active_entry = None;
                }
                widget.borrow_mut().state.set_active(!cur_state);
                (parent_list_box.cb)(parent_list_box.active_entry());
            })));
            children.push(widget)
        }

        children
    }
}
