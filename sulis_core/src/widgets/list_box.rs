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
use std::fmt::Display;
use std::rc::Rc;
use std::slice::Iter;

use crate::ui::{animation_state, AnimationState, Callback, Widget, WidgetKind};
use crate::widgets::Button;

#[derive(Clone)]
pub struct Entry<T: Display + Clone + 'static> {
    item: T,
    callback: Option<Callback>,
    animation_state: AnimationState,
}

impl<T: Display + Clone + 'static> Entry<T> {
    pub fn item(&self) -> &T {
        &self.item
    }
}

impl<T: Display + Clone + 'static> Entry<T> {
    pub fn new(item: T, callback: Option<Callback>) -> Entry<T> {
        Entry {
            item,
            callback,
            animation_state: AnimationState::default(),
        }
    }

    pub fn with_state(
        item: T,
        callback: Option<Callback>,
        animation_state: AnimationState,
    ) -> Entry<T> {
        Entry {
            item,
            callback,
            animation_state,
        }
    }

    pub fn with_active(item: T, callback: Option<Callback>) -> Entry<T> {
        Entry {
            item,
            callback,
            animation_state: AnimationState::with(animation_state::Kind::Active),
        }
    }
}

pub struct ListBox<T: Display + Clone + 'static> {
    pub(crate) entries: Vec<Entry<T>>,
}

impl<T: Display + Clone + 'static> ListBox<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<RefCell<ListBox<T>>> {
        Rc::new(RefCell::new(ListBox { entries }))
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries.get(index)
    }
}

pub const NAME: &str = "list_box";

impl<T: Display + Clone> WidgetKind for ListBox<T> {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children: Vec<Rc<RefCell<Widget>>> = Vec::with_capacity(self.entries.len());

        for entry in self.entries.iter() {
            let widget = Widget::with_theme(Button::with_text(&entry.item.to_string()), "entry");
            widget
                .borrow_mut()
                .state
                .set_animation_state(&entry.animation_state);
            if let Some(ref cb) = entry.callback {
                widget.borrow_mut().state.add_callback(cb.clone());
            }
            children.push(widget);
        }

        children
    }
}
