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
extern crate sulis_rules;
extern crate sulis_module;
extern crate sulis_state;
extern crate sulis_widgets;

extern crate chrono;
#[macro_use] extern crate log;

mod abilities_bar;
pub use self::abilities_bar::AbilitiesBar;

mod ability_pane;
pub use self::ability_pane::AbilityPane;

mod action_kind;
pub use self::action_kind::ActionKind;

mod ap_bar;
pub use self::ap_bar::ApBar;

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

mod cutscene_window;
pub use self::cutscene_window::CutsceneWindow;

mod dialog_window;
pub use self::dialog_window::DialogWindow;

mod entity_mouseover;
pub use self::entity_mouseover::EntityMouseover;

mod game_over_menu;
pub use self::game_over_menu::GameOverMenu;

mod initiative_ticker;
pub use self::initiative_ticker::InitiativeTicker;

mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod item_action_menu;
pub use self::item_action_menu::ItemActionMenu;

mod item_button;
pub use self::item_button::ItemButton;

mod loading_screen;
pub use self::loading_screen::LoadingScreen;

mod merchant_window;
pub use self::merchant_window::MerchantWindow;

mod portrait_view;
pub use self::portrait_view::PortraitView;

mod prop_mouseover;
pub use self::prop_mouseover::PropMouseover;

mod prop_window;
pub use self::prop_window::PropWindow;

pub mod main_menu;

pub mod character_selector;

mod race_pane;
pub use self::race_pane::RacePane;

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::io::InputAction;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_state::{ChangeListener, GameState};
use sulis_widgets::{Button, ConfirmationWindow};

const NAME: &str = "root";

pub struct RootView {
}

impl RootView {
    pub fn new() -> Rc<RefCell<RootView>> {
        Rc::new(RefCell::new(RootView { }))
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
            MerchantWindow::new(merchant_id)
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_prop_window(&mut self, widget: &Rc<RefCell<Widget>>,
                              desired_state: bool, prop_index: usize) {
        self.set_window(widget, self::prop_window::NAME, desired_state, &|| {
            PropWindow::new(prop_index)
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::inventory_window::NAME, desired_state, &|| {
            InventoryWindow::new(&GameState::selected())
        });
    }

    pub fn set_character_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::character_window::NAME, desired_state, &|| {
            CharacterWindow::new(&GameState::selected())
        });
    }

    fn set_window(&mut self, widget: &Rc<RefCell<Widget>>, name: &str, desired_state: bool,
                     cb: &Fn() -> Rc<RefCell<WidgetKind>>) {
        match Widget::get_child_with_name(widget, name) {
            None => {
                if desired_state {
                    let window = Widget::with_defaults(cb());
                    Widget::add_child_to(&widget, window);
                }
            }, Some(ref window) => {
                if !desired_state {
                    window.borrow_mut().mark_for_removal();
                }
            }
        }
    }

    pub fn toggle_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::inventory_window::NAME);
        self.set_inventory_window(widget, desired_state);
    }

    pub fn toggle_character_window(&mut self, widget: &Rc<RefCell<Widget>>) {
        let desired_state = !Widget::has_child_with_name(widget, self::character_window::NAME);
        self.set_character_window(widget, desired_state);
    }

    pub fn show_menu(&mut self, widget: &Rc<RefCell<Widget>>) {
        let exit_window = Widget::with_theme(
            ConfirmationWindow::new(Callback::with(
                    Box::new(|| { GameState::set_exit(); }))),
                    "exit_confirmation_window");
        exit_window.borrow_mut().state.set_modal(true);
        Widget::add_child_to(&widget, exit_window);
    }

    pub fn end_turn(&self) {
        if GameState::is_pc_current() {
            let area_state = GameState::area_state();
            area_state.borrow_mut().turn_timer.next();
        }
    }
}

impl WidgetKind for RootView {
    widget_kind!(NAME);

    fn update(&mut self, widget: &Rc<RefCell<Widget>>) {
        match GameState::check_get_ui_callback() {
            None => (),
            Some(ui_cb) => {
                dialog_window::activate(widget, &ui_cb.on_trigger, &ui_cb.parent, &ui_cb.target);
            }
        }
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            ShowMenu => {
                self.show_menu(widget);
            },
            ToggleInventory => {
                let desired_state = !Widget::has_child_with_name(widget, self::inventory_window::NAME);
                self.set_inventory_window(widget, desired_state);
            },
            ToggleCharacter => {
                let desired_state = !Widget::has_child_with_name(widget, self::character_window::NAME);
                self.set_character_window(widget, desired_state);
            },
            EndTurn => {
                self.end_turn();
            },
            _ => return false,
        }

        true
    }

    fn on_remove(&mut self) {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.remove(NAME);

        let pc = GameState::selected();
        pc.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to root widget.");

        let area_widget = Widget::with_defaults(AreaView::new());

        let right_pane = Widget::with_defaults(RightPane::new());

        let bot_pane = Widget::empty("bottom_pane");
        {
            let end_turn_button = Widget::with_theme(Button::empty(), "end_turn_button");
            end_turn_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let root = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&root);
                view.end_turn();
            })));
            end_turn_button.borrow_mut().state.set_enabled(GameState::is_pc_current());

            let end_turn_button_ref = Rc::clone(&end_turn_button);
            let area_state = GameState::area_state();
            area_state.borrow_mut().turn_timer.listeners.add(
                ChangeListener::new(NAME, Box::new(move |timer| {
                    let enabled = match timer.current() {
                        None => false,
                        Some(entity) => entity.borrow().is_pc(),
                    };
                    end_turn_button_ref.borrow_mut().state.set_enabled(enabled);
                })));

            Widget::add_child_to(&bot_pane, end_turn_button);

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
            map_button.borrow_mut().state.set_enabled(false);

            let log_button = Widget::with_theme(Button::empty(), "log_button");
            log_button.borrow_mut().state.set_enabled(false);

            let men_button = Widget::with_theme(Button::empty(), "menu_button");
            men_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
                let parent = Widget::get_root(&widget);
                let view = Widget::downcast_kind_mut::<RootView>(&parent);
                view.show_menu(&parent);
            })));

            let abilities = Widget::with_defaults(AbilitiesBar::new(GameState::selected()));

            Widget::add_children_to(&bot_pane, vec![inv_button, cha_button, map_button,
                                    log_button, men_button, abilities]);
        }

        let widget_ref = Rc::clone(&widget);
        let pc = GameState::selected();
        pc.borrow_mut().actor.listeners.add(ChangeListener::new(NAME, Box::new(move |pc| {
            // TODO handle party death
            if pc.is_dead() {
                let menu = Widget::with_defaults(GameOverMenu::new());
                Widget::add_child_to(&widget_ref, menu);
            }
        })));

        let ap_bar = Widget::with_defaults(ApBar::new(pc));

        let ticker = Widget::with_defaults(InitiativeTicker::new());

        // area widget must be the first entry in the children list
        vec![area_widget, right_pane, bot_pane, ap_bar, ticker]
    }
}

pub struct RightPane {}

impl RightPane {
    pub fn new() -> Rc<RefCell<RightPane>> {
        Rc::new(RefCell::new(RightPane {}))
    }
}

impl WidgetKind for RightPane {
    widget_kind!("right_pane");

    fn on_remove(&mut self) { }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        GameState::add_party_listener(ChangeListener::invalidate("right_pane", &widget));

        let mut children = Vec::new();

        for entity in GameState::party() {
            let portrait = Widget::with_defaults(PortraitView::new(entity));
            children.push(portrait);
        }

        children
    }
}
