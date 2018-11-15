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

use sulis_core::ui::{Widget, WidgetKind, theme, Callback};
use sulis_core::io::{event};
use sulis_widgets::{Label, TextArea};
use sulis_module::{Actor, OnTrigger, MerchantData, Conversation,
    conversation::{Response}, Module, on_trigger};
use sulis_state::{EntityState, ChangeListener, GameState, ItemState,
    area_feedback_text::ColorKind, NextGameStep, script::entity_with_id};

use {character_window, CutsceneWindow, RootView, GameOverWindow, LoadingScreen,
    window_fade, WindowFade, ConfirmationWindow};

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

    // TODO support finding an old modal when a new one is removed to allow for this
    // to work and still keep the dialog window modal
    // fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
    //     let root = Widget::get_root(widget);
    //     let view = Widget::downcast_kind_mut::<RootView>(&root);
    //
    //     use sulis_core::io::InputAction::*;
    //     match key {
    //         ShowMenu => view.show_menu(&root),
    //         Exit => view.show_exit(&root),
    //         _ => return false,
    //     }
    //
    //     true
    // }

    fn on_remove(&mut self) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.entity.borrow().actor.actor.name);

        let cur_text = self.convo.text(&self.cur_node);
        let responses = self.convo.responses(&self.cur_node);

        let node_widget = Widget::with_theme(self.node.clone(), "node");
        {
            let entity = self.entity.borrow();
            for (ref flag, ref val) in entity.custom_flags() {
                node_widget.borrow_mut().state.add_text_arg(flag, val);
            }
        }
        node_widget.borrow_mut().state.add_text_arg("player_name", &self.pc.borrow().actor.actor.name);
        let cur_text = theme::expand_text_args(cur_text, &node_widget.borrow().state);

        if responses.is_empty() {
            widget.borrow_mut().mark_for_removal();

            let area_state = GameState::area_state();
            area_state.borrow_mut().add_feedback_text(cur_text, &self.entity,
                                                      ColorKind::Info);
            return Vec::new();
        }

        self.node.borrow_mut().text = Some(cur_text);

        activate(widget, self.convo.on_view(&self.cur_node), &self.pc, &self.entity);

        let responses_widget = Widget::empty("responses");
        {
            for response in responses {
                if !is_viewable(response, &self.pc, &self.entity) { continue; }

                let response_button = ResponseButton::new(&response, &self.pc);
                let widget = Widget::with_defaults(response_button);
                Widget::add_child_to(&responses_widget, widget);
            }
        }

        vec![title, node_widget, responses_widget]
    }
}

struct ResponseButton {
    text: String,
    to: Option<String>,
    on_select: Vec<OnTrigger>,
    pc: Rc<RefCell<EntityState>>,
}

impl ResponseButton {
    fn new(response: &Response, pc: &Rc<RefCell<EntityState>>) -> Rc<RefCell<ResponseButton>> {
        Rc::new(RefCell::new(ResponseButton {
            text: response.text.to_string(),
            to: response.to.clone(),
            on_select: response.on_select.clone(),
            pc: Rc::clone(&pc),
        }))
    }
}

impl WidgetKind for ResponseButton {
    widget_kind!("response_button");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let text_area = TextArea::empty();
        let text_area_widget = Widget::with_defaults(text_area.clone());

        text_area_widget.borrow_mut().state
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

        let parent = Widget::go_up_tree(widget, 2);
        let window = Widget::downcast_kind_mut::<DialogWindow>(&parent);

        activate(widget, &self.on_select, &window.pc, &window.entity);

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
            &target, ColorKind::Info);
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

        if is_match(on_trigger, pc, entity) { break; }
    }

    cur_node.to_string()
}

pub fn is_match(on_trigger: &Vec<OnTrigger>, pc: &Rc<RefCell<EntityState>>,
                target: &Rc<RefCell<EntityState>>) -> bool {
    for trigger in on_trigger.iter() {
        use sulis_module::OnTrigger::*;
        match trigger {
            PlayerCoins(amount) => {
                let cur = GameState::party_coins();
                if cur < *amount { return false; }
            },
            PartyMember(ref id) => {
                if !GameState::has_party_member(id) { return false; }
            },
            PartyItem(ref id) => {
                let stash = GameState::party_stash();
                if !stash.borrow().has_item(id) { return false; }
            },
            TargetNumFlag(ref data) => {
                if target.borrow().get_num_flag(&data.flag) < data.val { return false; }
            },
            PlayerNumFlag(ref data) => {
                if pc.borrow().get_num_flag(&data.flag) < data.val { return false; }
            },
            NotTargetNumFlag(ref data) => {
                if target.borrow().get_num_flag(&data.flag) >= data.val { return false; }
            },
            NotPlayerNumFlag(ref data) => {
                if pc.borrow().get_num_flag(&data.flag) >= data.val { return false; }
            },
            NotTargetFlag(ref flag) => {
                if target.borrow().has_custom_flag(flag) { return false; }
            },
            NotPlayerFlag(ref flag) => {
                if pc.borrow().has_custom_flag(flag) { return false; }
            },
            TargetFlag(ref flag) => {
                if !target.borrow().has_custom_flag(flag) { return false; }
            },
            PlayerFlag(ref flag) => {
                if !pc.borrow().has_custom_flag(flag) { return false; }
            },
            PlayerAbility(ref ability_to_find) => {
                let mut has_ability = false;
                for ability in pc.borrow().actor.actor.abilities.iter() {
                    if &ability.ability.id == ability_to_find {
                        has_ability = true;
                        break;
                    }
                }

                if !has_ability { return false; }
            },
            _ => {
                warn!("Unsupported OnTrigger kind '{:?}' in validator", trigger);
            }
        }
    }

    true
}

pub fn is_viewable(response: &Response, pc: &Rc<RefCell<EntityState>>,
                   target: &Rc<RefCell<EntityState>>) -> bool {
    is_match(&response.to_view, pc, target)
}

pub fn activate(widget: &Rc<RefCell<Widget>>, on_select: &Vec<OnTrigger>,
                pc: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {

    use sulis_module::OnTrigger::*;
    for trigger in on_select.iter() {
        match trigger {
            PlayerAbility(ref ability_id) => {
                let ability = match Module::ability(ability_id) {
                    None => {
                        warn!("No ability found for '{}' when activating on_trigger", ability_id);
                        return;
                    }, Some(ability) => ability,
                };

                let mut pc = pc.borrow_mut();
                let state = &mut pc.actor;
                let new_actor = Actor::from(&state.actor, None, state.actor.xp, vec![ability],
                                            state.actor.inventory.clone());
                state.replace_actor(new_actor);
            },
            PlayerCoins(amount) => {
                GameState::add_party_coins(*amount);
            },
            PartyMember(ref id) => {
                match entity_with_id(id.to_string()) {
                    None => warn!("Attempted to add party member '{}' but entity does not exist",
                                  id),
                    Some(entity) => GameState::add_party_member(entity),
                }
            },
            PartyItem(ref id) => {
                let stash = GameState::party_stash();
                match ItemState::from(id) {
                    None => warn!("Attempted to add item '{}' but it does not exist", id),
                    Some(item) => { stash.borrow_mut().add_item(1, item); },
                }
            },
            TargetNumFlag(ref data) => {
                target.borrow_mut().add_num_flag(&data.flag, data.val);
            },
            PlayerNumFlag(ref data) => {
                pc.borrow_mut().add_num_flag(&data.flag, data.val);
            },
            NotTargetNumFlag(ref data) => {
                target.borrow_mut().clear_custom_flag(&data.flag);
            },
            NotPlayerNumFlag(ref data) => {
                pc.borrow_mut().clear_custom_flag(&data.flag);
            },
            NotTargetFlag(ref flag) => {
                target.borrow_mut().clear_custom_flag(flag);
            },
            NotPlayerFlag(ref flag) => {
                pc.borrow_mut().clear_custom_flag(flag);
            },
            TargetFlag(ref flag) => {
                target.borrow_mut().set_custom_flag(flag, "true");
            },
            PlayerFlag(ref flag) => {
                pc.borrow_mut().set_custom_flag(flag, "true");
            },
            ShowMerchant(ref merch) => show_merchant(widget, merch),
            StartConversation(ref convo) => start_convo(widget, convo, pc, target),
            SayLine(ref line) => {
                let area_state = GameState::area_state();
                area_state.borrow_mut().add_feedback_text(line.to_string(), &target,
                ColorKind::Info);
            },
            ShowCutscene(ref cutscene) => show_cutscene(widget, cutscene),
            FireScript(ref script) => fire_script(&script.id, &script.func, pc, target),
            GameOverWindow(ref text) => game_over_window(widget, text.to_string()),
            ScrollView(x, y) => scroll_view(widget, *x, *y),
            LoadModule(ref module_id) => load_module(widget, module_id),
            ShowConfirm(ref data) => show_confirm(widget, data),
            FadeOutIn => fade_out_in(widget),
        }
    }
}

fn show_confirm(widget: &Rc<RefCell<Widget>>, data: &on_trigger::DialogData) {
    let root = Widget::get_root(widget);

    let cb = if let Some(ref on_accept) = data.on_accept {
        let id = on_accept.id.to_string();
        let func = on_accept.func.to_string();
        Callback::new(Rc::new(move |widget, _| {
            let target = GameState::player();
            fire_script(&id, &func, &target, &target);

            let parent = Widget::get_parent(&widget);
            parent.borrow_mut().mark_for_removal();
        }))
    } else {
        Callback::empty()
    };
    let window = ConfirmationWindow::new(cb);
    {
        let title = Rc::clone(window.borrow().title());
        title.borrow_mut().state.add_text_arg("message", &data.message);
        window.borrow().add_accept_text_arg("text", &data.accept_text);
        window.borrow().add_cancel_text_arg("text", &data.cancel_text);
    }

    let widget = Widget::with_theme(window, "script_confirmation");
    widget.borrow_mut().state.set_modal(true);
    Widget::add_child_to(&root, widget);
}

fn load_module(widget: &Rc<RefCell<Widget>>, module_id: &str) {
    let root = Widget::get_root(widget);
    let view = Widget::downcast_kind_mut::<RootView>(&root);

    let pc = GameState::player();
    let inventory = character_window::get_inventory(&pc.borrow().actor);
    let actor = Actor::from(&pc.borrow().actor.actor, None, pc.borrow().actor.xp(),
        Vec::new(), inventory);

    let modules_list = Module::get_available_modules();
    for module in modules_list {
        if module.id != module_id { continue; }

        let step = NextGameStep::LoadModuleAndNewCampaign {
            pc_actor: Rc::new(actor),
            module_dir: module.dir.to_string(),
        };
        view.set_next_step(step);

        let loading_screen = Widget::with_defaults(LoadingScreen::new());
        loading_screen.borrow_mut().state.set_modal(true);
        Widget::add_child_to(&root, loading_screen);
        return;
    }

    warn!("Unable to load module '{}' as it was not found.", module_id);
}

fn fade_out_in(widget: &Rc<RefCell<Widget>>) {
    let root = Widget::get_root(widget);
    let (_, area_view_widget) = {
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.area_view()
    };

    let fade = Widget::with_defaults(WindowFade::new(window_fade::Mode::OutIn));

    Widget::add_child_to(&area_view_widget, fade);
}

fn scroll_view(widget: &Rc<RefCell<Widget>>, x: i32, y: i32) {
    let root = Widget::get_root(widget);

    let (area_view, area_view_widget) = {
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.area_view()
    };

    let (width, height) = {
        let area = GameState::area_state();
        let area = area.borrow();
        (area.area.width, area.area.height)
    };

    area_view.borrow_mut().delayed_scroll_to_point(x as f32, y as f32, width, height,
                                                   &area_view_widget.borrow());
}

fn game_over_window(widget: &Rc<RefCell<Widget>>, text: String) {
    let menu_cb = Callback::new(Rc::new(|widget, _| {
        let root = Widget::get_root(widget);
        let root_view = Widget::downcast_kind_mut::<RootView>(&root);
        root_view.next_step = Some(NextGameStep::MainMenu);
    }));
    let window = Widget::with_theme(GameOverWindow::new(menu_cb, text),
        "script_game_over_window");
    window.borrow_mut().state.set_modal(true);
    let root = Widget::get_root(widget);
    Widget::add_child_to(&root, window);
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

fn start_convo(widget: &Rc<RefCell<Widget>>, convo_id: &str, pc: &Rc<RefCell<EntityState>>,
               target: &Rc<RefCell<EntityState>>) {
    let convo = match Module::conversation(convo_id) {
        None => {
            warn!("Unable to find convo '{}' for on_trigger", convo_id);
            return;
        }, Some(convo) => convo,
    };

    info!("Showing conversation {}", convo_id);

    show_convo(convo, pc, target, widget);
}
