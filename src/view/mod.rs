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

mod action_menu;
pub use self::action_menu::ActionMenu;

mod area_view;
pub use self::area_view::AreaView;

mod character_window;
pub use self::character_window::CharacterWindow;

mod entity_mouseover;
pub use self::entity_mouseover::EntityMouseover;

mod game_over_menu;
pub use self::game_over_menu::GameOverMenu;

mod initiative_ticker;
pub use self::initiative_ticker::InitiativeTicker;

mod inventory_window;
pub use self::inventory_window::InventoryWindow;

use std::rc::Rc;
use std::cell::RefCell;

use grt::io::InputAction;
use grt::ui::{Button, Callback, ConfirmationWindow, EmptyWidget, Label, Widget, WidgetKind};
use state::{ChangeListener, GameState};

const NAME: &str = "root";

pub struct RootView {
}

impl RootView {
    pub fn new() -> Rc<RootView> {
        Rc::new(RootView {
        })
    }
}

impl WidgetKind for RootView {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use grt::io::InputAction::*;
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
                let window = Widget::get_child_with_name(widget,
                                self::inventory_window::NAME);
                match window {
                    None => {
                        let window = Widget::with_defaults(
                            InventoryWindow::new(&GameState::pc()));
                        Widget::add_child_to(&widget, window);
                    },
                    Some(window) => window.borrow_mut().mark_for_removal(),
                }
            },
            ToggleCharacter => {
                let window = Widget::get_child_with_name(widget,
                                                         self::character_window::NAME);
                match window {
                    None => {
                        let window = Widget::with_defaults(
                            CharacterWindow::new(&GameState::pc()));
                        Widget::add_child_to(&widget, window);
                    },
                    Some(window) => window.borrow_mut().mark_for_removal(),
                }
            },
            _ => return false,

        }

        true
    }

    fn on_remove(&self) {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.remove(NAME);

        let pc = GameState::pc();
        pc.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to root widget.");

        let mouse_over = Widget::with_theme(Label::empty(), "mouse_over");

        let area_widget = Widget::with_defaults(
            AreaView::new(Rc::clone(&mouse_over)));

        let right_pane = Widget::with_theme(EmptyWidget::new(), "right_pane");
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
