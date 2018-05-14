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
use sulis_module::{Actor, OnTrigger, MerchantData, Conversation, conversation::{Response}, Module};
use sulis_state::{EntityState, ChangeListener, GameState};
use sulis_core::ui::{Widget, WidgetKind, color};
use sulis_widgets::{Label, TextArea};

use {CutsceneWindow, RootView};

pub const NAME: &str = "dialog_window";

pub struct DialogWindow {
    pc: Rc<RefCell<EntityState>>,
    entity: Rc<RefCell<EntityState>>,
    convo: Rc<Conversation>,
    cur_node: String,

    node: Rc<RefCell<TextArea>>,
}

impl DialogWindow {
    pub fn new(pc: &Rc<RefCell<EntityState>>, entity: &Rc<RefCell<EntityState>>,
               convo: Rc<Conversation>) -> Rc<RefCell<DialogWindow>> {
        let cur_node = get_initial_node(&convo, pc, entity);

        Rc::new(RefCell::new(DialogWindow {
            pc: Rc::clone(pc),
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
        let node_widget = Widget::with_theme(self.node.clone(), "node");
        for flag in self.entity.borrow().custom_flags() {
            node_widget.borrow_mut().state.add_text_arg(flag, "true");
        }

        if let &Some(ref on_select) = self.convo.on_view(&self.cur_node) {
            activate(widget, on_select, &self.pc, &self.entity);
        }

        let responses = Widget::empty("responses");
        {
            for response in self.convo.responses(&self.cur_node) {
                if !is_viewable(response, &self.pc, &self.entity) { continue; }

                let response_button = ResponseButton::new(&response);
                let widget = Widget::with_defaults(response_button);
                Widget::add_child_to(&responses, widget);
            }
        }

        vec![title, node_widget, responses]
    }
}

struct ResponseButton {
    text: String,
    to: Option<String>,
    on_select: Option<OnTrigger>,
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
        let window = Widget::downcast_kind_mut::<DialogWindow>(&parent);

        if let Some(ref on_select) = self.on_select {
            activate(widget, on_select, &window.pc, &window.entity);
        }

        match self.to {
            None => {
                parent.borrow_mut().mark_for_removal();
            }, Some(ref to) => {
                window.cur_node = to.to_string();
                parent.borrow_mut().invalidate_children();
            }
        }

        true
    }
}

pub fn show_convo(convo: Rc<Conversation>, pc: &Rc<RefCell<EntityState>>,
                  target: &Rc<RefCell<EntityState>>, widget: &Rc<RefCell<Widget>>) {
    let initial_node = get_initial_node(&convo, &pc, &target);
    if convo.responses(&initial_node).is_empty() {
        let area_state = GameState::area_state();
        area_state.borrow_mut().add_feedback_text(convo.text(&initial_node).to_string(),
            &target, color::GRAY);
    } else {
        let window = Widget::with_defaults(DialogWindow::new(&pc, &target, convo));
        window.borrow_mut().state.set_modal(true);

        let root = Widget::get_root(widget);
        Widget::add_child_to(&root, window);
    }
}

pub fn get_initial_node(convo: &Rc<Conversation>, pc: &Rc<RefCell<EntityState>>,
                        entity: &Rc<RefCell<EntityState>>) -> String {
    let mut cur_node = "";
    for (node, on_trigger) in convo.initial_nodes() {
        cur_node = node;

        if match on_trigger {
            Some(on_trigger) => is_match(on_trigger, pc, entity),
                None => true,
        } {
            break
        }
    }

    cur_node.to_string()
}

pub fn is_match(on_trigger: &OnTrigger, pc: &Rc<RefCell<EntityState>>,
                target: &Rc<RefCell<EntityState>>) -> bool {
    if let Some(ref flags) = on_trigger.target_flags {
        for flag in flags.iter() {
            if !target.borrow_mut().has_custom_flag(flag) { return false; }
        }
    }

    if let Some(ref flags) = on_trigger.player_flags {
        for flag in flags.iter() {
            if !pc.borrow_mut().has_custom_flag(flag) { return false; }
        }
    }

    if let Some(ref ability_to_find) = on_trigger.player_ability {
        let mut has_ability = false;
        for ability in pc.borrow().actor.actor.abilities.iter() {
            if &ability.id == ability_to_find {
                has_ability = true;
                break;
            }
        }

        if !has_ability { return false; }
    }

    true
}

pub fn is_viewable(response: &Response, pc: &Rc<RefCell<EntityState>>,
                   target: &Rc<RefCell<EntityState>>) -> bool {
    if let Some(ref on_select) = response.to_view {
        is_match(on_select, pc, target)
    } else {
        true
    }
}

pub fn activate(widget: &Rc<RefCell<Widget>>, on_select: &OnTrigger,
                pc: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {
    if let Some(ref ability_id) = on_select.player_ability {
        let ability = match Module::ability(ability_id) {
            None => {
                warn!("No ability found for '{}' when activating on_trigger", ability_id);
                return;
            }, Some(ability) => ability,
        };

        let mut pc = pc.borrow_mut();
        let state = &mut pc.actor;
        let new_actor = Actor::from(&state.actor, None, state.actor.xp, vec![ability]);
        state.replace_actor(new_actor);
    }

    if let Some(ref flags) = on_select.target_flags {
        for flag in flags.iter() {
            target.borrow_mut().set_custom_flag(flag);
        }
    }

    if let Some(ref flags) = on_select.player_flags {
        for flag in flags.iter() {
            pc.borrow_mut().set_custom_flag(flag);
        }
    }

    if let Some(ref merch) = on_select.show_merchant {
        show_merchant(widget, merch);
    }

    if let Some(ref convo) = on_select.start_conversation {
        start_convo(widget, convo);
    }

    if let Some(ref cutscene) = on_select.show_cutscene {
        show_cutscene(widget, cutscene);
    }

    if let Some(ref script) = on_select.fire_script {
        fire_script(&script.id, &script.func, pc, target)
    }
}

fn fire_script(script_id: &str, func: &str, parent: &Rc<RefCell<EntityState>>,
               target: &Rc<RefCell<EntityState>>) {
    GameState::execute_trigger_script(script_id, func, parent, target);
}

fn show_merchant(widget: &Rc<RefCell<Widget>>, merch: &MerchantData) {
    let id = &merch.id;
    let loot = match Module::loot_list(&merch.loot_list) {
        None => {
            warn!("Unable to find loot list '{}' for merchant '{}'", merch.loot_list, id);
            return;
        }, Some(loot) => loot,
    };

    {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        area_state.get_or_create_merchant(id, &loot, merch.buy_frac, merch.sell_frac);
    }

    let root = Widget::get_root(widget);
    let root_view = Widget::downcast_kind_mut::<RootView>(&root);
    root_view.set_merchant_window(&root, true, &id);
}

fn show_cutscene(widget: &Rc<RefCell<Widget>>, cutscene_id: &str) {
    let cutscene = match Module::cutscene(cutscene_id) {
        None => {
            warn!("Unable to find cutscene '{}' for on_trigger", cutscene_id);
            return;
        }, Some(cutscene) => cutscene,
    };

    info!("Showing cutscene '{}' with {} frames.", cutscene_id, cutscene.frames.len());

    let root = Widget::get_root(widget);
    let window = Widget::with_defaults(CutsceneWindow::new(cutscene));
    window.borrow_mut().state.set_modal(true);
    Widget::add_child_to(&root, window);
}

fn start_convo(widget: &Rc<RefCell<Widget>>, convo_id: &str) {
    let convo = match Module::conversation(convo_id) {
        None => {
            warn!("Unable to find convo '{}' for on_trigger", convo_id);
            return;
        }, Some(convo) => convo,
    };

    info!("Showing conversation {}", convo_id);

    let pc = GameState::pc();
    show_convo(convo, &pc, &pc, widget);
}
