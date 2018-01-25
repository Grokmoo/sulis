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

use std::fmt::Display;
use std::rc::Rc;
use std::slice::Iter;
use std::cell::RefCell;

use sulis_engine::ui::{AnimationState, Callback, Widget, WidgetKind};
use sulis_engine::util::Size;
use Button;

pub struct Entry<T: Display> {
    item: T,
    callback: Option<Callback>,
    animation_state: AnimationState,
}

impl<T: Display> Entry<T> {
    pub fn item(&self) -> &T {
        &self.item
    }
}

impl<T: Display> Entry<T> {
    pub fn new(item: T, callback: Option<Callback>) -> Entry<T> {
       Entry {
           item,
           callback,
           animation_state: AnimationState::default(),
       }
    }

    pub fn with_state(item: T, callback: Option<Callback>,
                      animation_state: AnimationState) -> Entry<T> {
        Entry {
            item,
            callback,
            animation_state,
        }
    }
}

pub struct ListBox<T: Display> {
    entries: Vec<Entry<T>>,
}

impl<T: Display> ListBox<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<ListBox<T>> {
        Rc::new(ListBox {
            entries,
        })
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries.get(index)
    }
}

impl<T: Display> WidgetKind for ListBox<T> {
    fn get_name(&self) -> &str {
        "list_box"
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_self_layout();

        let width = widget.state.inner_size.width;
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            let theme = match child.borrow().theme {
                None => continue,
                Some(ref t) => Rc::clone(t),
            };
            let height = theme.preferred_size.height;
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
        }
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children: Vec<Rc<RefCell<Widget>>> =
            Vec::with_capacity(self.entries.len());

        for entry in self.entries.iter() {
            let widget = Widget::with_theme(Button::with_text(&entry.item.to_string()), "entry");
            widget.borrow_mut().state.set_animation_state(&entry.animation_state);
            if let Some(ref cb) = entry.callback {
                widget.borrow_mut().state.add_callback(cb.clone());
            }
            children.push(widget);
        }

        children
    }
}
