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
use sulis_core::widgets::{Button, Label, TextArea};

const NAME: &str = "game_over_window";

pub struct GameOverWindow {
    menu_callback: Callback,
    content_text: String,
}

impl GameOverWindow {
    pub fn new(menu_callback: Callback, content_text: String) -> Rc<RefCell<GameOverWindow>> {
        Rc::new(RefCell::new(GameOverWindow { menu_callback, content_text }))
    }
}

impl WidgetKind for GameOverWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let label = Widget::with_theme(Label::empty(), "title");
        let content = Widget::with_theme(TextArea::new(&self.content_text), "content");
        let exit = Widget::with_theme(Button::empty(), "exit");
        exit.borrow_mut().state.add_callback(self.menu_callback.clone());

        vec![label, exit, content]
    }
}
