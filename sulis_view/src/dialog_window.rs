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

use sulis_core::io::event;
use sulis_module::Conversation;
use sulis_state::{EntityState, ChangeListener};
use sulis_core::ui::{Widget, WidgetKind};
use sulis_widgets::{Label, TextArea};

pub const NAME: &str = "dialog_window";

pub struct DialogWindow {
    entity: Rc<RefCell<EntityState>>,
    convo: Rc<Conversation>,
    cur_node: String,

    node: Rc<RefCell<TextArea>>,
}

impl DialogWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>, convo: Rc<Conversation>) -> Rc<RefCell<DialogWindow>> {
        let cur_node = convo.initial_node();

        Rc::new(RefCell::new(DialogWindow {
            entity: Rc::clone(entity),
            convo: convo,
            node: TextArea::empty(),
            cur_node,
        }))
    }
}

impl WidgetKind for DialogWindow {
    widget_kind!(NAME);

    fn on_remove(&mut self) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.entity.borrow().actor.actor.name);

        self.node.borrow_mut().text = Some(self.convo.text(&self.cur_node).to_string());

        let responses = Widget::empty("responses");
        {
            for response in self.convo.responses(&self.cur_node) {
                let response_button = ResponseButton::new(&response.text, &response.to);
                let widget = Widget::with_defaults(response_button);
                Widget::add_child_to(&responses, widget);
            }
        }

        vec![title, Widget::with_theme(self.node.clone(), "node"), responses]
    }
}

struct ResponseButton {
    text: String,
    to: Option<String>,
}

impl ResponseButton {
    fn new(text: &str, to: &Option<String>) -> Rc<RefCell<ResponseButton>> {
        Rc::new(RefCell::new(ResponseButton {
            text: text.to_string(),
            to: to.clone(),
        }))
    }
}

impl WidgetKind for ResponseButton {
    widget_kind!("response_button");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let text_area = Widget::with_defaults(TextArea::new(&self.text));

        vec![text_area]
    }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_self_layout();

        widget.do_children_layout();
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        let parent = Widget::go_up_tree(widget, 2);
        match self.to {
            None => {
                parent.borrow_mut().mark_for_removal();
            }, Some(ref to) => {
                let window = Widget::downcast_kind_mut::<DialogWindow>(&parent);
                window.cur_node = to.to_string();
                parent.borrow_mut().invalidate_children();
            }
        }

        true
    }
}
