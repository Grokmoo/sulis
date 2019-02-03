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

use crate::widget_kind;
use crate::widgets::{Button, Label};
use crate::ui::{Callback, Widget, WidgetKind};

pub struct ConfirmationWindow {
    accept_callback: Callback,
    title: Rc<RefCell<Widget>>,
    accept: Rc<RefCell<Widget>>,
    cancel: Rc<RefCell<Widget>>,
}

impl ConfirmationWindow {
    pub fn new(accept_callback: Callback) -> Rc<RefCell<ConfirmationWindow>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let cancel = Widget::with_theme(Button::empty(), "cancel");
        let accept = Widget::with_theme(Button::empty(), "accept");

        Rc::new(RefCell::new(ConfirmationWindow {
            accept_callback,
            title,
            accept,
            cancel,
        }))
    }

    pub fn add_accept_text_arg(&self, key: &str, value: &str) {
        self.accept.borrow_mut().state.add_text_arg(key, value);
    }

    pub fn add_cancel_text_arg(&self, key: &str, value: &str) {
        self.cancel.borrow_mut().state.add_text_arg(key, value);
    }

    pub fn title(&self) -> &Rc<RefCell<Widget>> {
        &self.title
    }
}

impl WidgetKind for ConfirmationWindow {
    widget_kind!("confirmation_window");

    fn layout(&mut self, widget: &mut Widget) {
        if widget.state.has_text_arg("custom") {
            let value = widget.state.get_text_arg("custom").unwrap();
            self.title.borrow_mut().state.add_text_arg("custom", value);
        }

        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let (parent, _) = Widget::parent::<ConfirmationWindow>(widget);
            parent.borrow_mut().mark_for_removal();
        })));
        self.accept.borrow_mut().state.add_callback(self.accept_callback.clone());

        vec![Rc::clone(&self.cancel), Rc::clone(&self.accept), Rc::clone(&self.title)]
    }
}
