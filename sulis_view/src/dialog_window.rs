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
use std::rc::Rc;

use sulis_core::io::{event, InputActionKind};
use sulis_core::ui::{theme, Widget, WidgetKind};
use sulis_core::widgets::TextArea;
use sulis_module::{conversation::Response, Conversation, OnTrigger};
use sulis_state::{
    area_feedback_text::ColorKind, script::entity_with_id, AreaFeedbackText, ChangeListener,
    EntityState, GameState,
};

use crate::trigger_activator::{activate, is_match, scroll_view};
use crate::{AreaView, RootView};

pub const NAME: &str = "dialog_window";

pub struct DialogWindow {
    pc: Rc<RefCell<EntityState>>,
    entity: Rc<RefCell<EntityState>>,
    convo: Rc<Conversation>,
    cur_node: String,

    node: Rc<RefCell<TextArea>>,
}

impl DialogWindow {
    pub fn new(
        pc: &Rc<RefCell<EntityState>>,
        entity: &Rc<RefCell<EntityState>>,
        convo: Rc<Conversation>,
    ) -> Rc<RefCell<DialogWindow>> {
        let cur_node = get_initial_node(&convo, pc, entity);

        Rc::new(RefCell::new(DialogWindow {
            pc: Rc::clone(pc),
            entity: Rc::clone(entity),
            convo,
            node: TextArea::empty(),
            cur_node,
        }))
    }
}

impl WidgetKind for DialogWindow {
    widget_kind!(NAME);

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputActionKind) -> bool {
        let (root, view) = Widget::parent_mut::<RootView>(widget);

        use sulis_core::io::InputActionKind::*;
        match key {
            Back => view.show_menu(&root),
            Exit => view.show_exit(&root),
            _ => return false,
        }

        true
    }

    fn on_remove(&mut self, _widget: &Rc<RefCell<Widget>>) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity
            .borrow_mut()
            .actor
            .listeners
            .add(ChangeListener::invalidate(NAME, widget));

        let cur_text = self.convo.text(&self.cur_node);
        let responses = self.convo.responses(&self.cur_node);

        let node_widget = Widget::with_theme(self.node.clone(), "node");
        {
            let node = &mut node_widget.borrow_mut().state;
            let entity = self.entity.borrow();
            for (flag, val) in entity.custom_flags() {
                node.add_text_arg(flag, val);
            }

            node.add_text_arg("player_name", &self.pc.borrow().actor.actor.name);
            node.add_text_arg("target_name", &entity.actor.actor.name);
        }

        let cur_text = theme::expand_text_args(cur_text, &node_widget.borrow().state);

        if responses.is_empty() {
            widget.borrow_mut().mark_for_removal();

            let area = GameState::area_state();
            let mut feedback = AreaFeedbackText::with_target(&self.entity.borrow(), &area.borrow());
            feedback.add_entry(cur_text, ColorKind::Info);
            area.borrow_mut().add_feedback_text(feedback);
            return Vec::new();
        }

        self.node.borrow_mut().text = Some(cur_text);

        activate(
            widget,
            self.convo.on_view(&self.cur_node),
            &self.pc,
            &self.entity,
        );

        let responses_widget = Widget::empty("responses");
        {
            for response in responses {
                if !is_viewable(response, &self.pc, &self.entity) {
                    continue;
                }

                let response_button = ResponseButton::new(&self.convo, response, &self.pc);
                let widget = Widget::with_defaults(response_button);
                Widget::add_child_to(&responses_widget, widget);
            }
        }

        vec![node_widget, responses_widget]
    }
}

struct ResponseButton {
    text: String,
    to: Option<String>,
    on_select: Vec<OnTrigger>,
    pc: Rc<RefCell<EntityState>>,
    convo: Rc<Conversation>,
}

impl ResponseButton {
    fn new(
        convo: &Rc<Conversation>,
        response: &Response,
        pc: &Rc<RefCell<EntityState>>,
    ) -> Rc<RefCell<ResponseButton>> {
        Rc::new(RefCell::new(ResponseButton {
            text: response.text.to_string(),
            to: response.to.clone(),
            on_select: response.on_select.clone(),
            pc: Rc::clone(pc),
            convo: Rc::clone(convo),
        }))
    }

    fn check_switch_speaker(&self, node: &str, area: &Rc<RefCell<AreaView>>) {
        let speaker = match self.convo.switch_speaker(node) {
            None => return,
            Some(ref speaker) => speaker,
        };

        let speaker = match entity_with_id(speaker.to_string()) {
            None => {
                warn!("Attempted to switch to invalid speaker '{}'", speaker);
                return;
            }
            Some(speaker) => speaker,
        };

        let (x, y) = {
            let speaker = &speaker.borrow().location;
            (speaker.x, speaker.y)
        };
        let cb = OnTrigger::ScrollView(x, y);
        GameState::add_ui_callback(vec![cb], &self.pc, &speaker);
        area.borrow_mut()
            .set_active_entity(Some(Rc::clone(&speaker)));
    }
}

impl WidgetKind for ResponseButton {
    widget_kind!("response_button");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let text_area = TextArea::empty();
        let text_area_widget = Widget::with_defaults(text_area.clone());

        text_area_widget
            .borrow_mut()
            .state
            .add_text_arg("player_name", &self.pc.borrow().actor.actor.name);
        let cur_text = theme::expand_text_args(&self.text, &text_area_widget.borrow().state);

        text_area.borrow_mut().text = Some(cur_text);
        vec![text_area_widget]
    }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_self_layout();

        widget.do_children_layout();
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        let (parent, window) = Widget::parent_mut::<DialogWindow>(widget);

        activate(widget, &self.on_select, &window.pc, &window.entity);

        let (_, view) = Widget::parent_mut::<RootView>(&parent);
        let (area, _) = view.area_view();

        match self.to {
            None => {
                parent.borrow_mut().mark_for_removal();
                area.borrow_mut().set_active_entity(None);
            }
            Some(ref to) => {
                self.check_switch_speaker(to, &area);
                window.cur_node = to.to_string();
                parent.borrow_mut().invalidate_children()
            }
        }

        true
    }
}

pub fn show_convo(
    convo: Rc<Conversation>,
    pc: &Rc<RefCell<EntityState>>,
    target: &Rc<RefCell<EntityState>>,
    widget: &Rc<RefCell<Widget>>,
) {
    let initial_node = get_initial_node(&convo, pc, target);
    if convo.responses(&initial_node).is_empty() {
        let area = GameState::area_state();

        let mut feedback = AreaFeedbackText::with_target(&target.borrow(), &area.borrow());
        feedback.add_entry(convo.text(&initial_node).to_string(), ColorKind::Info);
        area.borrow_mut().add_feedback_text(feedback);
    } else {
        let window = Widget::with_defaults(DialogWindow::new(pc, target, convo));
        window.borrow_mut().state.set_modal(true);

        let (root, view) = Widget::parent_mut::<RootView>(widget);
        let (area, _) = view.area_view();
        area.borrow_mut().clear_mouse_state();
        area.borrow_mut()
            .set_active_entity(Some(Rc::clone(target)));

        let (x, y) = {
            let loc = &target.borrow().location;
            (loc.x, loc.y)
        };
        scroll_view(&root, x, y);
        Widget::add_child_to(&root, window);
    }
}

pub fn get_initial_node(
    convo: &Rc<Conversation>,
    pc: &Rc<RefCell<EntityState>>,
    entity: &Rc<RefCell<EntityState>>,
) -> String {
    let mut cur_node = "";
    for (node, on_trigger) in convo.initial_nodes() {
        cur_node = node;

        if is_match(on_trigger, pc, entity) {
            break;
        }
    }

    cur_node.to_string()
}

pub fn is_viewable(
    response: &Response,
    pc: &Rc<RefCell<EntityState>>,
    target: &Rc<RefCell<EntityState>>,
) -> bool {
    is_match(&response.to_view, pc, target)
}
