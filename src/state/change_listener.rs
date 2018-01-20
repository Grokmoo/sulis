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

use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::Widget;

pub struct ChangeListenerList<T> {
   listeners: Vec<ChangeListener<T>>,
}

impl<T> Default for ChangeListenerList<T> {
    fn default() -> ChangeListenerList<T> {
        ChangeListenerList {
            listeners: Vec::new(),
        }
    }
}

impl<T> ChangeListenerList<T> {
    pub fn add(&mut self, listener: ChangeListener<T>) {
        self.remove(listener.id);
        self.listeners.push(listener);
    }

    pub fn remove(&mut self, id: &'static str) {
        self.listeners.retain(|listener| listener.id() != id);
    }

    pub fn notify(&self, t: &T) {
        for listener in self.listeners.iter() {
            listener.call(t);
        }
    }
}

pub struct ChangeListener<T> {
    cb: Box<Fn(&T)>,
    id: &'static str,
}

impl<T> ChangeListener<T> {
    pub fn new(id: &'static str, cb: Box<Fn(&T)>) -> ChangeListener<T> {
        ChangeListener {
            cb,
            id,
        }
    }

    pub fn remove_widget(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().mark_for_removal();
            }),
            id,
        }
    }

    pub fn invalidate(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().invalidate_children();
            }),
            id,
        }
    }

    pub fn invalidate_layout(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().invalidate_layout();
            }),
            id,
        }
    }

    pub fn call(&self, t: &T) {
        (self.cb)(t);
    }

    pub fn id(&self) -> &'static str {
        self.id
    }
}
