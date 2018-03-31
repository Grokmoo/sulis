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

pub mod character_builder;
pub use self::character_builder::CharacterBuilder;

mod character_window;
pub use self::character_window::CharacterWindow;

mod class_pane;
pub use self::class_pane::ClassPane;

mod entity_mouseover;
pub use self::entity_mouseover::EntityMouseover;

mod game_over_menu;
pub use self::game_over_menu::GameOverMenu;

mod initiative_ticker;
pub use self::initiative_ticker::InitiativeTicker;

mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod loading_screen;
pub use self::loading_screen::LoadingScreen;

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

    pub fn set_prop_window(&mut self, widget: &Rc<RefCell<Widget>>,
                              desired_state: bool, prop_index: usize) {
        self.set_window(widget, self::prop_window::NAME, desired_state, &|| {
            PropWindow::new(prop_index)
        });

        self.set_inventory_window(widget, desired_state);
    }

    pub fn set_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::inventory_window::NAME, desired_state, &|| {
            InventoryWindow::new(&GameState::pc())
        });
    }

    pub fn set_character_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.set_window(widget, self::character_window::NAME, desired_state, &|| {
            CharacterWindow::new(&GameState::pc())
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
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

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

        let pc = GameState::pc();
        pc.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to root widget.");

        let area_widget = Widget::with_defaults(AreaView::new());

        let right_pane = Widget::empty("right_pane");
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

            Widget::add_child_to(&right_pane, end_turn_button);

            let ticker = Widget::with_defaults(InitiativeTicker::new());
            Widget::add_child_to(&right_pane, ticker);

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

            let portrait = Widget::with_defaults(PortraitView::new(GameState::pc()));
            let abilities = Widget::with_defaults(AbilitiesBar::new(GameState::pc()));
            Widget::add_children_to(&right_pane, vec![inv_button, cha_button, map_button,
                                    log_button, men_button, portrait, abilities]);
        }

        let widget_ref = Rc::clone(&widget);
        let pc = GameState::pc();
        pc.borrow_mut().actor.listeners.add(ChangeListener::new(NAME, Box::new(move |pc| {
            if pc.is_dead() {
                let menu = Widget::with_defaults(GameOverMenu::new());
                Widget::add_child_to(&widget_ref, menu);
            }
        })));

        let ap_bar = Widget::with_defaults(ApBar::new(pc));

        vec![area_widget, right_pane, ap_bar]
    }
}
