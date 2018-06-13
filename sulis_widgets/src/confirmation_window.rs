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

use {Button, Label};
use sulis_core::ui::{Callback, Widget, WidgetKind};

pub struct ConfirmationWindow {
    accept_callback: Callback,
    title: Rc<RefCell<Widget>>,
}

impl ConfirmationWindow {
    pub fn new(accept_callback: Callback) -> Rc<RefCell<ConfirmationWindow>> {
        let title = Widget::with_theme(Label::empty(), "title");

        Rc::new(RefCell::new(ConfirmationWindow {
            accept_callback,
            title,
        }))
    }

    pub fn title(&self) -> &Rc<RefCell<Widget>> {
        &self.title
    }
}

impl WidgetKind for ConfirmationWindow {
    widget_kind!("confirmation_window");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::remove_parent());

        let accept = Widget::with_theme(Button::empty(), "accept");
        accept.borrow_mut().state.add_callback(self.accept_callback.clone());

        vec![cancel, accept, Rc::clone(&self.title)]
    }
}
