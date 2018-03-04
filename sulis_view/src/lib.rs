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

extern crate sulis_core;
extern crate sulis_rules;
extern crate sulis_module;
extern crate sulis_state;
extern crate sulis_widgets;

extern crate chrono;
#[macro_use] extern crate log;

mod action_menu;
pub use self::action_menu::ActionMenu;

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
use sulis_widgets::{Button, ConfirmationWindow, Label};

const NAME: &str = "root";

pub struct RootView {
}

impl RootView {
    pub fn new() -> Rc<RefCell<RootView>> {
        Rc::new(RefCell::new(RootView { }))
    }

    pub fn toggle_prop_window(&mut self, widget: &Rc<RefCell<Widget>>,
                              desired_state: bool, prop_index: usize) {
        self.toggle_window(widget, self::prop_window::NAME, desired_state, &|| {
            PropWindow::new(prop_index)
        });

        self.toggle_inventory_window(widget, desired_state);
    }

    pub fn toggle_inventory_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.toggle_window(widget, self::inventory_window::NAME, desired_state, &|| {
            InventoryWindow::new(&GameState::pc())
        });
    }

    pub fn toggle_character_window(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.toggle_window(widget, self::character_window::NAME, desired_state, &|| {
            CharacterWindow::new(&GameState::pc())
        });
    }

    pub fn toggle_character_builder(&mut self, widget: &Rc<RefCell<Widget>>, desired_state: bool) {
        self.toggle_window(widget, self::character_builder::NAME, desired_state, &|| {
            CharacterBuilder::new()
        });
    }

    fn toggle_window(&mut self, widget: &Rc<RefCell<Widget>>, name: &str, desired_state: bool,
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
}

impl WidgetKind for RootView {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            Exit => {
                let exit_window = Widget::with_theme(
                    ConfirmationWindow::new(Callback::with(
                            Box::new(|| { GameState::set_exit(); }))),
                    "exit_confirmation_window");
                exit_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&widget, exit_window);
            },
            ToggleInventory => {
                let desired_state = !Widget::has_child_with_name(widget, self::inventory_window::NAME);
                self.toggle_inventory_window(widget, desired_state);
            },
            ToggleCharacter => {
                let desired_state = !Widget::has_child_with_name(widget, self::character_window::NAME);
                self.toggle_character_window(widget, desired_state);
            },
            ToggleCharacterBuilder => {
                let desired_state = !Widget::has_child_with_name(widget, self::character_builder::NAME);
                self.toggle_character_builder(widget, desired_state);
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

        let mouse_over = Widget::with_theme(Label::empty(), "mouse_over");

        let area_widget = Widget::with_defaults(
            AreaView::new(Rc::clone(&mouse_over)));

        let right_pane = Widget::empty("right_pane");
        {
            let button = Widget::with_theme(Button::empty(), "end_turn_button");
            button.borrow_mut().state.add_callback(Callback::with(Box::new(|| {
                if GameState::is_pc_current() {
                    let area_state = GameState::area_state();
                    area_state.borrow_mut().turn_timer.next();
                }
            })));
            button.borrow_mut().state.set_enabled(GameState::is_pc_current());

            let button_ref = Rc::clone(&button);
            let area_state = GameState::area_state();
            area_state.borrow_mut().turn_timer.listeners.add(
                ChangeListener::new(NAME, Box::new(move |timer| {
                    let enabled = match timer.current() {
                        None => false,
                        Some(entity) => entity.borrow().is_pc(),
                    };
                    button_ref.borrow_mut().state.set_enabled(enabled);
                })));

            let area_state = GameState::area_state();
            let area_title = Widget::with_theme(
                Label::new(&area_state.borrow().area.name), "title");
            Widget::add_child_to(&right_pane, mouse_over);
            Widget::add_child_to(&right_pane, button);
            Widget::add_child_to(&right_pane, area_title);

            let ticker = Widget::with_defaults(InitiativeTicker::new());
            Widget::add_child_to(&right_pane, ticker);
        }

        let widget_ref = Rc::clone(&widget);
        let pc = GameState::pc();
        pc.borrow_mut().actor.listeners.add(ChangeListener::new(NAME, Box::new(move |pc| {
            if pc.is_dead() {
                let menu = Widget::with_defaults(GameOverMenu::new());
                Widget::add_child_to(&widget_ref, menu);
            }
        })));

        vec![area_widget, right_pane]
    }
}
