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
use sulis_widgets::{Button, TextArea, ScrollPane};
use sulis_state::script::{CallbackData, ScriptCallback, ScriptMenuSelection};

const NAME: &str = "script_menu";

pub struct ScriptMenu {
    callback: CallbackData,
    title: String,
    choices: Vec<String>,
}

impl ScriptMenu {
    pub fn new(callback: CallbackData, title: String,
               choices: Vec<String>) -> Rc<RefCell<ScriptMenu>> {
        Rc::new(RefCell::new(ScriptMenu { callback, title, choices }))
    }
}

impl WidgetKind for ScriptMenu {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let cancel= Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();
        })));

        let scrollpane = ScrollPane::new();
        let entries = Widget::with_theme(scrollpane.clone(), "entries");
        for choice in self.choices.iter() {
            let text_area = Widget::with_defaults(TextArea::empty());
            text_area.borrow_mut().state.add_text_arg("choice", choice);

            let widget = Widget::with_theme(Button::empty(), "entry");

            let text = choice.to_string();
            let cb = self.callback.clone();
            widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let selection = ScriptMenuSelection { value: text.to_string() };
                cb.on_menu_select(selection);

                let parent = Widget::go_up_tree(&widget, 3);
                parent.borrow_mut().mark_for_removal();
            })));
            Widget::add_child_to(&widget, text_area);
            scrollpane.borrow().add_to_content(widget);
        }

        let title = Widget::with_theme(TextArea::empty(), "title");
        title.borrow_mut().state.add_text_arg("title", &self.title);

        vec![cancel, title, entries]
    }
}
