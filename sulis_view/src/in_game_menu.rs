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
use sulis_widgets::{Button, ConfirmationWindow};

const NAME: &str = "in_game_menu";

pub struct InGameMenu {
    exit_callback: Callback,
    menu_callback: Callback,
}

impl InGameMenu {
    pub fn new(exit_callback: Callback, menu_callback: Callback) -> Rc<RefCell<InGameMenu>> {
        Rc::new(RefCell::new(InGameMenu { exit_callback, menu_callback }))
    }
}

impl WidgetKind for InGameMenu {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let back = Widget::with_theme(Button::empty(), "back");
        back.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();
        })));

        let menu = Widget::with_theme(Button::empty(), "menu");
        let menu_cb = self.menu_callback.clone();
        menu.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let window = Widget::with_theme(ConfirmationWindow::new(menu_cb.clone()), "menu_confirmation");
            window.borrow_mut().state.set_modal(true);

            let root = Widget::get_root(widget);
            Widget::add_child_to(&root, window);
        })));

        let exit = Widget::with_theme(Button::empty(), "exit");
        let exit_cb = self.exit_callback.clone();
        exit.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let root = Widget::get_root(widget);

            let window = Widget::with_theme(ConfirmationWindow::new(exit_cb.clone()), "exit_confirmation");
            window.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, window);
        })));

        vec![back, menu, exit]
    }
}
