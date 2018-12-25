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
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, TextArea};
use sulis_module::{Ability, Conversation, Module, conversation::Response, OnTrigger};

use crate::{CharacterBuilder};
use crate::character_builder::BuilderPane;

pub const NAME: &str = "backstory_selector_pane";

pub struct BackstorySelectorPane {
    node: Rc<RefCell<TextArea>>,
    complete: bool,
    cur_node: String,
    convo: Rc<Conversation>,
    abilities: Vec<Rc<Ability>>,
}

pub fn get_initial_node(convo: &Rc<Conversation>) -> String {
    let mut cur_node = "";
    for (node, _) in convo.initial_nodes() {
        cur_node = node;
        break;
    }

    cur_node.to_string()
}

impl BackstorySelectorPane {
    pub fn new() -> Rc<RefCell<BackstorySelectorPane>> {
        let convo = Rc::clone(&Module::campaign().backstory_conversation);
        let cur_node = get_initial_node(&convo);
        Rc::new(RefCell::new(BackstorySelectorPane {
            node: TextArea::empty(),
            cur_node,
            convo,
            complete: false,
            abilities: Vec::new(),
        }))
    }

    pub fn set_next_enabled(&mut self, widget: &Rc<RefCell<Widget>>) {
        let builder_widget = Widget::get_parent(&widget);
        let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);

        let next = self.complete || self.convo.responses(&self.cur_node).len() == 0;

        builder.finish.borrow_mut().state.set_enabled(next);
    }
}

impl BuilderPane for BackstorySelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.abilities.clear();

        let next = self.complete || self.convo.responses(&self.cur_node).len() == 0;
        builder.finish.borrow_mut().state.set_enabled(next);
        builder.next.borrow_mut().state.set_visible(false);
        builder.finish.borrow_mut().state.set_visible(true);
        widget.borrow_mut().invalidate_children();
    }

    // since this is the last pane, this is called before save
    fn next(&mut self, builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
        for ability in self.abilities.iter() {
            builder.abilities.push(Rc::clone(ability));
        }
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        self.cur_node = get_initial_node(&self.convo);
        self.abilities.clear();
        builder.next.borrow_mut().state.set_visible(true);
        builder.finish.borrow_mut().state.set_visible(false);
        builder.prev(&widget);
    }
}

impl WidgetKind for BackstorySelectorPane {
    widget_kind!(NAME);

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        self.node.borrow_mut().text = Some(self.convo.text(&self.cur_node).to_string());
        let node_widget = Widget::with_theme(self.node.clone(), "node");
        let responses = Widget::empty("responses");
        {
            for response in self.convo.responses(&self.cur_node) {
                let response_button = ResponseButton::new(&response);
                let widget = Widget::with_defaults(response_button);
                Widget::add_child_to(&responses, widget);
            }
        }

        let start_over = Widget::with_theme(Button::empty(), "start_over");
        start_over.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let pane = Widget::downcast_kind_mut::<BackstorySelectorPane>(&parent);
            pane.cur_node = get_initial_node(&pane.convo);
            pane.abilities.clear();
            pane.set_next_enabled(&parent);

            parent.borrow_mut().invalidate_children();
        })));

        vec![title, node_widget, responses, start_over]
   }
}

struct ResponseButton {
    text: String,
    to: Option<String>,
    on_select: Vec<OnTrigger>,
}

impl ResponseButton {
    fn new(response: &Response) -> Rc<RefCell<ResponseButton>> {
        Rc::new(RefCell::new(ResponseButton {
            text: response.text.to_string(),
            to: response.to.clone(),
            on_select: response.on_select.clone(),
        }))
    }
}

impl WidgetKind for ResponseButton {
    widget_kind!("response_button");

    fn on_add(&mut self, _: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        vec![Widget::with_defaults(TextArea::new(&self.text))]
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        let parent = Widget::go_up_tree(widget, 2);
        let pane = Widget::downcast_kind_mut::<BackstorySelectorPane>(&parent);

        for trigger in self.on_select.iter() {
            match trigger {
                OnTrigger::PlayerAbility(ability_id) => {
                    match Module::ability(ability_id) {
                        None => {
                            warn!("No ability found for '{}' in backstory", ability_id);
                        }, Some(ability) => pane.abilities.push(ability),
                    }
                },
                _ => {
                    warn!("Unsupported OnTrigger variant '{:?}' in backstory", trigger);
                }
            }
        }

        match self.to {
            None => {
                pane.complete = true;
            }, Some(ref to) => {
                pane.cur_node = to.to_string();
            }
        }

        pane.set_next_enabled(&parent);
        parent.borrow_mut().invalidate_children();
        true
    }
}
