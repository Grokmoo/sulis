//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::collections::HashMap;
use std::{any::Any, cell::RefCell, rc::Rc, time::Instant};

use crate::{
    character_window, formation_window, inventory_window, merchant_window, prop_window,
    quest_window, world_map_window, AbilitiesBar, ApBar, AreaView, CharacterWindow, ConsoleWindow,
    FormationWindow, GameOverWindow, InGameMenu, InitiativeTicker, InventoryWindow, MerchantWindow,
    PortraitPane, PropWindow, QuestWindow, QuickItemBar, WorldMapWindow,
};
use sulis_core::config::Config;
use sulis_core::io::{keyboard_event::Key, InputAction};
use sulis_core::ui::{Callback, Cursor, Scrollable, Widget, WidgetKind};
use sulis_core::util;
use sulis_core::widgets::{Button, ConfirmationWindow, Label};
use sulis_module::{area::OnRest, Module};
use sulis_state::{
    area_feedback_text::ColorKind, save_file::create_save, script::script_callback,
    script::ScriptEntity, AreaFeedbackText, ChangeListener, EntityState, GameState, NextGameStep,
    Script,
};

const WINDOW_NAMES: [&str; 7] = [
    self::formation_window::NAME,
    self::inventory_window::NAME,
    self::character_window::NAME,
    self::quest_window::NAME,
    self::world_map_window::NAME,
    self::merchant_window::NAME,
    self::prop_window::NAME,
];

const NAME: &str = "game";

pub struct RootView {
    pub(crate) next_step: Option<NextGameStep>,
    status: Rc<RefCell<Widget>>,
    status_added: Option<Instant>,
    area_view: Rc<RefCell<AreaView>>,
    area_view_widget: Rc<RefCell<Widget>>,
    console: Rc<RefCell<ConsoleWindow>>,
    console_widget: Rc<RefCell<Widget>>,

    quick_item_bar: Option<Rc<RefCell<Widget>>>,
    abilities_bar: Option<Rc<RefCell<Widget>>>,
    area: String,
}

impl RootView {
    pub fn area_view(&self) -> (Rc<RefCell<AreaView>>, Rc<RefCell<Widget>>) {
        (
            Rc::clone(&self.area_view),
            Rc::clone(&self.area_view_widget),
        )
    }

    pub fn next_step(&mut self) -> Option<NextGameStep> {
        self.next_step.take()
    }

    pub fn set_next_step(&mut self, step: NextGameStep) {
        self.next_step = Some(step);
    }

    pub fn add_status_text(&mut self, text: &str) {
        self.status.borrow_mut().state.text = text.to_string();
        self.status_added = Some(Instant::now());
    }

    pub fn new() -> Rc<RefCell<RootView>> {
        let area_view = AreaView::new(Scrollable::default());
        let area_view_widget = Widget::with_defaults(area_view.clone());

        let console = ConsoleWindow::new();
        let console_widget = Widget::with_defaults(console.clone());

        Rc::new(RefCell::new(RootView {
            next_step: None,
            status: Widget::with_theme(Label::empty(), "status_text"),
            status_added: None,
            area_view,
            area_view_widget,
            area: "".to_string(),
            console,
            console_widget,
            quick_item_bar: None,
            abilities_bar: None,
        }))
    }

    /// Gets the merchant window if it is currently opened
    pub fn get_merchant_window(&self, widget: &Rc<RefCell<Widget>>) -> Option<Rc<RefCell<Widget>>> {
        match Widget::get_child_with_name(widget, merchant_window::NAME) {
            None => None,
            Some(ref window) => Some(Rc::clone(window)),
        }
    }

    /// Gets the prop window if it is currently opened
    pub fn get_prop_window(&self, widget: &Rc<RefCell<Widget>>) -> Option<Rc<RefCell<Widget>>> {
        match Widget::get_child_with_name(widget, prop_window::NAME) {
            None => None,
            Some(ref window) => Some(Rc::clone(window)),
        }
    }

    pub fn set_merchant_window(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        desired_state: bool,
        merchant_id: &str,
    ) {
        self.set_window(widget, self::merchant_window::NAME, desired_state, &|| {
            match GameState::selected().first() {
                None => None,
                Some(ref entity) => Some(MerchantWindow::new(merchant_id, Rc::clone(entity))),
            }
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_prop_window(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        desired_state: bool,
        prop_index: usize,
    ) {
        self.set_window(widget, self::prop_window::NAME, desired_state, &|| {
            match GameState::selected().first() {
                None => None,
                Some(ref entity) => Some(PropWindow::new(prop_index, Rc::clone(entity))),
            }
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::inventory_window::NAME, desired_state, &|| {
            match GameState::selected().first() {
                None => None,
                Some(ref entity) => Some(InventoryWindow::new(entity)),
            }
        });
    }

    pub fn set_character_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::character_window::NAME, desired_state, &|| {
            match GameState::selected().first() {
                None => None,
                Some(ref entity) => Some(CharacterWindow::new(entity)),
            }
        });
    }

    pub fn set_console_window(&mut self, _widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.console_widget
            .borrow_mut()
            .state
            .set_visible(desired_state);
        if desired_state {
            self.console.borrow().grab_keyboard();
        }
    }

    pub fn set_quest_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::quest_window::NAME, desired_state, &|| {
            Some(QuestWindow::new())
        });
    }

    pub fn set_formation_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::formation_window::NAME, desired_state, &|| {
            Some(FormationWindow::new())
        });
    }

    pub fn set_map_window(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        desired_state: bool,
        transition_enabled: bool,
    ) {
        self.set_window(widget, self::world_map_window::NAME, desired_state, &|| {
            Some(WorldMapWindow::new(transition_enabled))
        });
    }

    fn set_window(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        name: &str,
        desired_state: bool,
        cb: &dyn Fn() -> Option<Rc<RefCell<dyn WidgetKind>>>,
    ) {
        match Widget::get_child_with_name(widget, name) {
            None => {
                if desired_state {
                    let widget_kind = match cb() {
                        None => return,
                        Some(kind) => kind,
                    };
                    let window = Widget::with_defaults(widget_kind);
                    Widget::add_child_to(&widget, window);
                }
            }
            Some(ref window) => {
                if !desired_state {
                    window.borrow_mut().mark_for_removal();
                }
            }
        }
    }

    // returns true if there was at least one open window, false otherwise
    fn close_all_windows(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        let mut found = false;
        for name in &WINDOW_NAMES {
            match Widget::get_child_with_name(widget, name) {
                None => continue,
                Some(ref window) => {
                    window.borrow_mut().mark_for_removal();
                    found = true;
                }
            }
        }

        found
    }

    pub fn toggle_formation_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::formation_window::NAME);
        self.set_formation_window(widget, desired_state);
    }

    pub fn toggle_console_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !self.console_widget.borrow().state.is_visible();
        self.set_console_window(widget, desired_state);
    }

    pub fn toggle_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::inventory_window::NAME);
        self.set_inventory_window(widget, desired_state);
    }

    pub fn toggle_character_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::character_window::NAME);
        self.set_character_window(widget, desired_state);
    }

    pub fn toggle_quest_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::quest_window::NAME);
        self.set_quest_window(widget, desired_state);
    }

    pub fn toggle_map_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::world_map_window::NAME);
        self.set_map_window(widget, desired_state, false);
    }

    pub fn show_menu(&mut self, widget: &Rc<RefCell<Widget>>) {
        let exit_cb = Callback::new(Rc::new(|widget, _| {
            let (_, root_view) = Widget::parent_mut::<RootView>(widget);
            root_view.next_step = Some(NextGameStep::Exit);
        }));

        let menu_cb = Callback::new(Rc::new(|widget, _| {
            let (_, root_view) = Widget::parent_mut::<RootView>(widget);
            root_view.next_step = Some(NextGameStep::MainMenu);
        }));

        let menu = Widget::with_defaults(InGameMenu::new(exit_cb, menu_cb));
        menu.borrow_mut().state.set_modal(true);
        Widget::add_child_to(&widget, menu);
    }

    pub fn show_exit(&mut self, widget: &Rc<RefCell<Widget>>) {
        let exit_cb = Callback::new(Rc::new(|widget, _| {
            let (_, view) = Widget::parent_mut::<RootView>(widget);
            view.next_step = Some(NextGameStep::Exit);
        }));

        let window = Widget::with_theme(ConfirmationWindow::new(exit_cb), "exit_confirmation");
        window.borrow_mut().state.set_modal(true);
        Widget::add_child_to(widget, window);
    }

    pub fn end_turn(&self) {
        self.cancel_targeter();

        if GameState::is_pc_current() {
            let mgr = GameState::turn_manager();
            let cbs = mgr.borrow_mut().next();
            script_callback::fire_round_elapsed(cbs);
        }
    }

    fn cancel_targeter(&self) {
        let area = GameState::area_state();
        let area = area.borrow();

        if let Some(targeter) = area.targeter() {
            targeter.borrow_mut().on_cancel();
        }
    }

    pub fn rest(&self) {
        let area_state = GameState::area_state();
        let area = Rc::clone(&area_state.borrow().area.area);

        let target = GameState::player();
        match area.on_rest {
            OnRest::Disabled { ref message } => {
                let mut feedback =
                    AreaFeedbackText::with_target(&target.borrow(), &area_state.borrow());
                feedback.add_entry(message.to_string(), ColorKind::Info);
                area_state.borrow_mut().add_feedback_text(feedback);
            }
            OnRest::FireScript { ref id, ref func } => {
                Script::trigger(id, func, ScriptEntity::from(&target));
            }
        }
    }

    pub fn save(&mut self) {
        if GameState::is_combat_active() {
            self.add_status_text("Cannot save during combat.");
            return;
        }

        if let Err(e) = create_save() {
            error!("Error quick saving game");
            error!("{}", e);
            self.add_status_text("Error performing Save!");
        } else {
            self.add_status_text("Save Complete.");
        }
    }

    pub fn select_party_member(&self, index: usize) {
        let party = GameState::party();

        if let Some(member) = party.get(index) {
            GameState::set_selected_party_member(Rc::clone(member));
        }
    }
}

impl WidgetKind for RootView {
    widget_kind!(NAME);

    fn update(&mut self, widget: &Rc<RefCell<Widget>>, millis: u32) {
        let area_state = GameState::area_state();
        let root = Widget::get_root(widget);
        let area = area_state.borrow().area.area.id.clone();
        if area != self.area {
            self.area = area;
            root.borrow_mut().invalidate_children();
        }

        if let Some(instant) = self.status_added {
            let elapsed = util::get_elapsed_millis(instant.elapsed());
            if elapsed > 5000 {
                self.status_added = None;
                self.status.borrow_mut().state.text = String::new();
            }
        }

        let root = Widget::get_root(widget);
        let has_modal = root.borrow().has_modal();
        GameState::set_modal_locked(has_modal);

        let (cx, cy) = (Cursor::get_x(), Cursor::get_y());
        let mut area_view_updated = false;
        if !has_modal {
            let len = root.borrow().children.len();
            for i in (0..len).rev() {
                let child = Rc::clone(&root.borrow().children[i]);
                {
                    let child = child.borrow();
                    if !child.state.is_enabled() || !child.state.visible {
                        continue;
                    }

                    if !child.state.in_bounds(cx, cy) {
                        continue;
                    }
                }

                if Rc::ptr_eq(&child, &self.area_view_widget) {
                    self.area_view
                        .borrow_mut()
                        .update_cursor_and_hover(&self.area_view_widget);
                    area_view_updated = true;
                }
                break;
            }
        }

        if !area_view_updated {
            self.area_view.borrow_mut().clear_area_mouseover();
        }

        if !has_modal && Config::edge_scrolling() {
            if cx == Config::ui_width() - 1 {
                self.area_view.borrow_mut().scroll(-2.0, 0.0, millis);
            } else if cx == 0 {
                self.area_view.borrow_mut().scroll(2.0, 0.0, millis);
            }

            if cy == Config::ui_height() - 1 {
                self.area_view.borrow_mut().scroll(0.0, -2.0, millis);
            } else if cy == 0 {
                self.area_view.borrow_mut().scroll(0.0, 2.0, millis);
            }
        }
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        trace!("Key press: {:?} in root view.", key);
        use sulis_core::io::InputAction::*;
        match key {
            Back => {
                if !self.close_all_windows(widget) {
                    self.show_menu(widget);
                }
            }
            ToggleConsole => self.toggle_console_window(widget),
            ToggleInventory => self.toggle_inventory_window(widget),
            ToggleCharacter => self.toggle_character_window(widget),
            ToggleMap => self.toggle_map_window(widget),
            ToggleJournal => self.toggle_quest_window(widget),
            ToggleFormation => self.toggle_formation_window(widget),
            EndTurn => self.end_turn(),
            Rest => self.rest(),
            Exit => self.show_exit(widget),
            SelectAll => GameState::select_party_members(GameState::party()),
            QuickSave => self.save(),
            ScrollUp => self.area_view.borrow_mut().scroll(0.0, 2.0, 33),
            ScrollDown => self.area_view.borrow_mut().scroll(0.0, -2.0, 33),
            ScrollRight => self.area_view.borrow_mut().scroll(-2.0, 0.0, 33),
            ScrollLeft => self.area_view.borrow_mut().scroll(2.0, 0.0, 33),
            SelectPartyMember1 => self.select_party_member(0),
            SelectPartyMember2 => self.select_party_member(1),
            SelectPartyMember3 => self.select_party_member(2),
            SelectPartyMember4 => self.select_party_member(3),
            _ => {
                if let Some(quick_item_bar) = &self.quick_item_bar {
                    let bar: &QuickItemBar = Widget::kind(&quick_item_bar);
                    if bar.check_handle_keybinding(key) {
                        return true;
                    }
                }

                if let Some(abilities_bar) = &self.abilities_bar {
                    let bar: &AbilitiesBar = Widget::kind(abilities_bar);
                    return bar.check_handle_keybinding(key);
                }
            }
        }

        true
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        info!("Adding to root widget.");

        let keys = Config::get_keybindings();
        use InputAction::*;

        self.console_widget.borrow_mut().state.set_visible(false);

        let prev_scroll = self.area_view.borrow().get_scroll();
        self.area_view = AreaView::new(prev_scroll);
        self.area_view_widget = Widget::with_defaults(self.area_view.clone());

        let bot_pane = Widget::empty("bottom_pane");
        {
            let portrait_pane = Widget::with_defaults(PortraitPane::new());

            let formations = create_button(
                &keys,
                ToggleFormation,
                "formations_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.toggle_formation_window(&root);
                }),
            );

            let select_all = create_button(
                &keys,
                SelectAll,
                "select_all_button",
                Rc::new(|_, _| {
                    GameState::select_party_members(GameState::party());
                }),
            );
            let rest = create_button(
                &keys,
                Rest,
                "rest_button",
                Rc::new(|widget, _| {
                    let (_, view) = Widget::parent_mut::<RootView>(widget);
                    view.rest();
                }),
            );

            let navi_pane = Widget::empty("navi_pane");

            let end_turn_button = create_button(
                &keys,
                EndTurn,
                "end_turn_button",
                Rc::new(|widget, _| {
                    let (_, view) = Widget::parent_mut::<RootView>(widget);
                    view.end_turn();
                }),
            );
            end_turn_button
                .borrow_mut()
                .state
                .set_enabled(GameState::is_pc_current());

            let end_turn_button_ref = Rc::clone(&end_turn_button);
            let mgr = GameState::turn_manager();
            mgr.borrow_mut().listeners.add(ChangeListener::new(
                NAME,
                Box::new(move |timer| {
                    let enabled = match timer.current() {
                        None => false,
                        Some(entity) => entity.borrow().is_party_member(),
                    };
                    end_turn_button_ref.borrow_mut().state.set_enabled(enabled);
                }),
            ));

            let inv_button = create_button(
                &keys,
                ToggleInventory,
                "inventory_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.toggle_inventory_window(&root);
                }),
            );

            let cha_button = create_button(
                &keys,
                ToggleCharacter,
                "character_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.toggle_character_window(&root);
                }),
            );

            let map_button = create_button(
                &keys,
                ToggleMap,
                "map_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.toggle_map_window(&root);
                }),
            );

            let log_button = create_button(
                &keys,
                ToggleJournal,
                "journal_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.toggle_quest_window(&root);
                }),
            );

            let men_button = create_button(
                &keys,
                Back,
                "menu_button",
                Rc::new(|widget, _| {
                    let (root, view) = Widget::parent_mut::<RootView>(widget);
                    view.show_menu(&root);
                }),
            );

            Widget::add_children_to(
                &navi_pane,
                vec![
                    end_turn_button,
                    inv_button,
                    cha_button,
                    map_button,
                    log_button,
                    men_button,
                ],
            );

            let mut selected = GameState::selected();
            let entity = if selected.is_empty() {
                GameState::player()
            } else {
                selected.remove(0)
            };

            let quick_items = Widget::with_defaults(QuickItemBar::new(&entity, &keys));
            self.quick_item_bar = Some(Rc::clone(&quick_items));

            let abilities = Widget::with_defaults(AbilitiesBar::new(entity, &keys));
            self.abilities_bar = Some(Rc::clone(&abilities));

            let time_label = Widget::with_theme(Label::empty(), "time");
            let mgr = GameState::turn_manager();
            let time = mgr.borrow().current_time();
            let rules = Module::rules();
            let hour = rules.get_hour_name(time.hour);
            time_label
                .borrow_mut()
                .state
                .add_text_arg("day", &time.day.to_string());
            time_label.borrow_mut().state.add_text_arg("hour", hour);
            time_label
                .borrow_mut()
                .state
                .add_text_arg("round", &time.round.to_string());

            let time_label_ref = Rc::clone(&time_label);
            mgr.borrow_mut().time_listeners.add(ChangeListener::new(
                NAME,
                Box::new(move |time| {
                    let rules = Module::rules();
                    let hour = rules.get_hour_name(time.hour);
                    time_label_ref
                        .borrow_mut()
                        .state
                        .add_text_arg("day", &time.day.to_string());
                    time_label_ref.borrow_mut().state.add_text_arg("hour", hour);
                    time_label_ref
                        .borrow_mut()
                        .state
                        .add_text_arg("round", &time.round.to_string());
                    time_label_ref.borrow_mut().invalidate_layout();
                }),
            ));

            mgr.borrow_mut().time_listeners.add(ChangeListener::new(
                "ambient",
                Box::new(|time| {
                    let area = GameState::area_state();
                    area.borrow().update_ambient_audio(time);
                }),
            ));

            Widget::add_children_to(
                &bot_pane,
                vec![
                    abilities,
                    quick_items,
                    navi_pane,
                    portrait_pane,
                    select_all,
                    formations,
                    rest,
                    time_label,
                ],
            );
        }

        let widget_ref = Rc::clone(widget);
        GameState::add_party_death_listener(ChangeListener::new(
            NAME,
            Box::new(move |party| {
                if !is_defeated(party) {
                    return;
                }

                // set player to disabled, even though they are actually dead
                // this prevents this callback from being called over and over
                party[0].borrow_mut().actor.set_disabled(true);

                let menu_cb = Callback::new(Rc::new(|widget, _| {
                    let (_, view) = Widget::parent_mut::<RootView>(widget);
                    view.next_step = Some(NextGameStep::MainMenu);
                }));
                let menu = Widget::with_defaults(GameOverWindow::new(menu_cb, String::new()));
                Widget::add_child_to(&widget_ref, menu);
            }),
        ));

        let ap_bar = Widget::with_defaults(ApBar::new(GameState::player()));

        let ticker = Widget::with_defaults(InitiativeTicker::new());

        // area widget must be the first entry in the children list
        vec![
            Rc::clone(&self.area_view_widget),
            bot_pane,
            ap_bar,
            ticker,
            self.status.clone(),
            Rc::clone(&self.console_widget),
        ]
    }
}

type CB = dyn Fn(&Rc<RefCell<Widget>>, &mut dyn WidgetKind);

fn create_button(
    keybindings: &HashMap<InputAction, Key>,
    action: InputAction,
    id: &str,
    cb: Rc<CB>,
) -> Rc<RefCell<Widget>> {
    let button = Widget::with_theme(Button::empty(), id);
    {
        let mut button = button.borrow_mut();
        button.state.add_callback(Callback::new(cb));
        if let Some(key) = keybindings.get(&action) {
            button.state.add_text_arg("keybinding", &key.short_name())
        }
    }

    button
}

fn is_defeated(party: &[Rc<RefCell<EntityState>>]) -> bool {
    if party.is_empty() {
        return true;
    }

    {
        let player = &party[0].borrow().actor;
        if player.is_dead() && !player.is_disabled() {
            return true;
        }
    }

    for member in party.iter() {
        if !member.borrow().actor.is_dead() {
            return false;
        }
    }

    true
}
