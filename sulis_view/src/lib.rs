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

#[macro_use] extern crate sulis_core;
#[macro_use] extern crate log;

mod abilities_bar;
pub use self::abilities_bar::AbilitiesBar;

mod ability_pane;
pub use self::ability_pane::AbilityPane;

mod action_kind;
pub use self::action_kind::ActionKind;

mod ap_bar;
pub use self::ap_bar::ApBar;

mod area_mouseover;
pub use self::area_mouseover::AreaMouseover;

mod area_view;
pub use self::area_view::AreaView;

mod basic_mouseover;
pub use self::basic_mouseover::BasicMouseover;

pub mod character_builder;
pub use self::character_builder::CharacterBuilder;

mod character_window;
pub use self::character_window::CharacterWindow;

mod class_pane;
pub use self::class_pane::ClassPane;

mod console_window;
pub use self::console_window::ConsoleWindow;

mod cutscene_window;
pub use self::cutscene_window::CutsceneWindow;

mod dialog_window;
pub use self::dialog_window::DialogWindow;

mod formation_window;
pub use self::formation_window::FormationWindow;

mod game_over_window;
pub use self::game_over_window::GameOverWindow;

mod in_game_menu;
pub use self::in_game_menu::InGameMenu;

mod initiative_ticker;
pub use self::initiative_ticker::InitiativeTicker;

mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod item_action_menu;
pub use self::item_action_menu::ItemActionMenu;

mod item_button;
pub use self::item_button::ItemButton;

mod item_list_pane;
pub use self::item_list_pane::ItemListPane;

mod loading_screen;
pub use self::loading_screen::LoadingScreen;

mod load_window;
pub use self::load_window::LoadWindow;

mod merchant_window;
pub use self::merchant_window::MerchantWindow;

mod portrait_view;
pub use self::portrait_view::PortraitView;

mod prop_window;
pub use self::prop_window::PropWindow;

mod quest_window;
pub use self::quest_window::QuestWindow;

mod quick_item_bar;
use self::quick_item_bar::QuickItemBar;

pub mod main_menu;

mod race_pane;
pub use self::race_pane::RacePane;

mod script_menu;
pub use self::script_menu::ScriptMenu;

mod window_fade;
pub use self::window_fade::WindowFade;

mod world_map_window;
pub use self::world_map_window::WorldMapWindow;

use std::time::Instant;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::config::Config;
use sulis_core::io::{InputAction, MainLoopUpdater};
use sulis_core::ui::{Callback, Widget, WidgetKind, Cursor};
use sulis_core::util;
use sulis_module::area::OnRest;
use sulis_state::{ChangeListener, GameState, NextGameStep,
    save_file::create_save, script::script_callback, area_feedback_text::ColorKind};
use sulis_widgets::{Button, ConfirmationWindow, Label};

const NAME: &str = "root";

pub struct GameMainLoopUpdater {
    view: Rc<RefCell<RootView>>,
}

impl GameMainLoopUpdater {
    pub fn new(view: &Rc<RefCell<RootView>>) -> GameMainLoopUpdater {
        GameMainLoopUpdater {
            view: Rc::clone(view),
        }
    }
}

impl MainLoopUpdater for GameMainLoopUpdater {
    fn update(&self, root: &Rc<RefCell<Widget>>, millis: u32) {
        let ui_cb = GameState::update(millis);

        if let Some(cb) = ui_cb {
            dialog_window::activate(root, &cb.on_trigger, &cb.parent, &cb.target);
        }
    }

    fn is_exit(&self) -> bool {
        self.view.borrow().next_step.is_some()
    }
}

pub struct RootView {
    next_step: Option<NextGameStep>,
    status: Rc<RefCell<Widget>>,
    status_added: Option<Instant>,
    area_view: Rc<RefCell<AreaView>>,
    area_view_widget: Rc<RefCell<Widget>>,
    console: Rc<RefCell<ConsoleWindow>>,
    console_widget: Rc<RefCell<Widget>>,
    area: String,
}

impl RootView {
    pub fn area_view(&self) -> (Rc<RefCell<AreaView>>, Rc<RefCell<Widget>>) {
        (Rc::clone(&self.area_view), Rc::clone(&self.area_view_widget))
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
        let area_view = AreaView::new();
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
        }))
    }

    /// Gets the merchant window if it is currently opened
    pub fn get_merchant_window(&self, widget: &Rc<RefCell<Widget>>) -> Option<Rc<RefCell<Widget>>> {
        match Widget::get_child_with_name(widget, self::merchant_window::NAME) {
            None => None,
            Some(ref window) => Some(Rc::clone(window)),
        }
    }

    /// Gets the prop window if it is currently opened
    pub fn get_prop_window(&self, widget: &Rc<RefCell<Widget>>) -> Option<Rc<RefCell<Widget>>> {
        match Widget::get_child_with_name(widget, self::prop_window::NAME) {
            None => None,
            Some(ref window) => Some(Rc::clone(window)),
        }
    }

    pub fn set_merchant_window(&mut self, widget: &Rc<RefCell<Widget>>,
                               desired_state: bool, merchant_id: &str) {
        self.set_window(widget, self::merchant_window::NAME, desired_state, &|| {
            match GameState::selected().first() {
                None => None,
                Some(ref entity) => Some(MerchantWindow::new(merchant_id, Rc::clone(entity))),
            }
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_prop_window(&mut self, widget: &Rc<RefCell<Widget>>,
                              desired_state: bool, prop_index: usize) {
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
        self.console_widget.borrow_mut().state.set_visible(desired_state);
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

    pub fn set_map_window(&mut self, widget: &Rc<RefCell<Widget>>,
                          desired_state: bool, transition_enabled: bool) {
        self.set_window(widget, self::world_map_window::NAME, desired_state, &|| {
            Some(WorldMapWindow::new(transition_enabled))
        });
    }

    fn set_window(&mut self, widget: &Rc<RefCell<Widget>>, name: &str, desired_state: bool,
                     cb: &Fn() -> Option<Rc<RefCell<WidgetKind>>>) {
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
            }, Some(ref window) => {
                if !desired_state {
                    window.borrow_mut().mark_for_removal();
                }
            }
        }
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
            let root = Widget::get_root(widget);
            let root_view = Widget::downcast_kind_mut::<RootView>(&root);
            root_view.next_step = Some(NextGameStep::Exit);
        }));

        let menu_cb = Callback::new(Rc::new(|widget, _| {
            let root = Widget::get_root(widget);
            let root_view = Widget::downcast_kind_mut::<RootView>(&root);
            root_view.next_step = Some(NextGameStep::MainMenu);
        }));

        let menu = Widget::with_defaults(InGameMenu::new(exit_cb, menu_cb));
        menu.borrow_mut().state.set_modal(true);
        Widget::add_child_to(&widget, menu);
    }

    pub fn show_exit(&mut self, widget: &Rc<RefCell<Widget>>) {
        let exit_cb = Callback::new(Rc::new(|widget, _| {
            let root = Widget::get_root(widget);
            let view = Widget::downcast_kind_mut::<RootView>(&root);
            view.next_step = Some(NextGameStep::Exit);
        }));

        let window = Widget::with_theme(ConfirmationWindow::new(exit_cb), "exit_confirmation");
        window.borrow_mut().state.set_modal(true);
        Widget::add_child_to(widget, window);
    }

    pub fn end_turn(&self) {
        if GameState::is_pc_current() {
            let mgr = GameState::turn_manager();
            let cbs = mgr.borrow_mut().next();
            script_callback::fire_round_elapsed(cbs);
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
}

impl WidgetKind for RootView {
    widget_kind!(NAME);

    fn update(&mut self, widget: &Rc<RefCell<Widget>>, millis: u32) {
        let area_state = GameState::area_state();
        let root = Widget::get_root(widget);
        let area = area_state.borrow().area.id.clone();
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
                    if !child.state.is_enabled() || !child.state.visible { continue; }

                    if !child.state.in_bounds(cx, cy) { continue; }
                }

                if Rc::ptr_eq(&child, &self.area_view_widget) {
                    self.area_view.borrow_mut().update_cursor_and_hover(&self.area_view_widget);
                    area_view_updated = true;
                }
                break;
            }
        }

        if !area_view_updated { self.area_view.borrow_mut().clear_area_mouseover(); }

        if Config::edge_scrolling() {
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
        use sulis_core::io::InputAction::*;
        match key {
            ShowMenu => self.show_menu(widget),
            ToggleConsole => self.toggle_console_window(widget),
            ToggleInventory => self.toggle_inventory_window(widget),
            ToggleCharacter => self.toggle_character_window(widget),
            ToggleMap => self.toggle_map_window(widget),
            ToggleJournal => self.toggle_quest_window(widget),
            ToggleFormation => self.toggle_formation_window(widget),
            EndTurn => self.end_turn(),
            Exit => self.show_exit(widget),
            SelectAll => GameState::select_party_members(GameState::party()),
            QuickSave => self.save(),
            ScrollUp => self.area_view.borrow_mut().scroll(0.0, 2.0, 33),
            ScrollDown => self.area_view.borrow_mut().scroll(0.0, -2.0, 33),
            ScrollRight => self.area_view.borrow_mut().scroll(-2.0, 0.0, 33),
            ScrollLeft => self.area_view.borrow_mut().scroll(2.0, 0.0, 33),
            _ => return false,
        }

        true
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        info!("Adding to root widget.");

        self.console_widget.borrow_mut().state.set_visible(false);

        self.area_view = AreaView::new();
        self.area_view_widget = Widget::with_defaults(self.area_view.clone());

        let bot_pane = Widget::empty("bottom_pane");
        {
            let portrait_pane = Widget::with_defaults(PortraitPane::new());

            let formations = Widget::with_theme(Button::empty(), "formations_button");
            formations.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.toggle_formation_window(&parent);
            })));

            let select_all = Widget::with_theme(Button::empty(), "select_all_button");
            select_all.borrow_mut().state.add_callback(Callback::new(Rc::new(|_, _| {
                GameState::select_party_members(GameState::party());
            })));

            let rest = Widget::with_theme(Button::empty(), "rest_button");
            rest.borrow_mut().state.add_callback(Callback::new(Rc::new(|_, _| {
                let area_state = GameState::area_state();
                let area = Rc::clone(&area_state.borrow().area);

                let target = GameState::player();
                match area.on_rest {
                    OnRest::Disabled { ref message } => {
                        area_state.borrow_mut().add_feedback_text(message.to_string(),
                            &target, ColorKind::Info);
                    },
                    OnRest::FireScript { ref id, ref func } => {
                        GameState::execute_trigger_script(id, func, &target, &target);
                    }
                }
            })));

            let navi_pane = Widget::empty("navi_pane");

            let end_turn_button = Widget::with_theme(Button::empty(), "end_turn_button");
            end_turn_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let root = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&root);
                view.end_turn();
            })));
            end_turn_button.borrow_mut().state.set_enabled(GameState::is_pc_current());

            let end_turn_button_ref = Rc::clone(&end_turn_button);
            let mgr = GameState::turn_manager();
            mgr.borrow_mut().listeners.add(
                ChangeListener::new(NAME, Box::new(move |timer| {
                    let enabled = match timer.current() {
                        None => false,
                        Some(entity) => entity.borrow().is_party_member(),
                    };
                    end_turn_button_ref.borrow_mut().state.set_enabled(enabled);
                })));

            let inv_button = Widget::with_theme(Button::empty(), "inventory_button");
            inv_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.toggle_inventory_window(&parent);
            })));

            let cha_button = Widget::with_theme(Button::empty(), "character_button");
            cha_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.toggle_character_window(&parent);
            })));

            let map_button = Widget::with_theme(Button::empty(), "map_button");
            map_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.toggle_map_window(&parent);
            })));

            let log_button = Widget::with_theme(Button::empty(), "journal_button");
            log_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.toggle_quest_window(&parent);
            })));

            let men_button = Widget::with_theme(Button::empty(), "menu_button");
            men_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.show_menu(&parent);
            })));

            Widget::add_children_to(&navi_pane, vec![end_turn_button, inv_button,
                                    cha_button, map_button, log_button, men_button]);

            let mut selected = GameState::selected();
            let entity = if selected.is_empty() {
                GameState::player()
            } else {
                selected.remove(0)
            };

            let quick_items = Widget::with_defaults(QuickItemBar::new(&entity));
            let abilities = Widget::with_defaults(AbilitiesBar::new(entity));

            Widget::add_children_to(&bot_pane, vec![abilities, quick_items, navi_pane,
                                    portrait_pane, select_all, formations, rest]);
        }

        let widget_ref = Rc::clone(widget);
        let pc = GameState::player();
        pc.borrow_mut().actor.listeners.add(ChangeListener::new(NAME, Box::new(move |pc| {
            if !pc.is_dead() { return; }
            let menu_cb = Callback::new(Rc::new(|widget, _| {
                let root = Widget::get_root(widget);
                let root_view = Widget::downcast_kind_mut::<RootView>(&root);
                root_view.next_step = Some(NextGameStep::MainMenu);
            }));
            let menu = Widget::with_defaults(GameOverWindow::new(menu_cb, String::new()));
            Widget::add_child_to(&widget_ref, menu);
        })));

        let ap_bar = Widget::with_defaults(ApBar::new(GameState::player()));

        let ticker = Widget::with_defaults(InitiativeTicker::new());

        // area widget must be the first entry in the children list
        vec![Rc::clone(&self.area_view_widget), bot_pane, ap_bar, ticker, self.status.clone(),
            Rc::clone(&self.console_widget)]
    }
}

pub struct PortraitPane {}

impl PortraitPane {
    pub fn new() -> Rc<RefCell<PortraitPane>> {
        Rc::new(RefCell::new(PortraitPane {}))
    }
}

impl WidgetKind for PortraitPane {
    widget_kind!("portrait_pane");

    fn on_remove(&mut self) { }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        GameState::add_party_listener(ChangeListener::invalidate("portrait_pane", &widget));

        let mut children = Vec::new();

        let selected = GameState::selected();
        for entity in GameState::party() {
            if !entity.borrow().show_portrait() { continue; }

            let mut is_selected = false;
            for sel in selected.iter() {
                if Rc::ptr_eq(sel, &entity) {
                    is_selected = true;
                    break;
                }
            }
            let portrait = Widget::with_defaults(PortraitView::new(entity));
            portrait.borrow_mut().state.set_active(is_selected);
            children.push(portrait);
        }

        children
    }
}
