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

#[macro_use]
extern crate sulis_core;
#[macro_use]
extern crate log;

mod area_overlay_handler;
pub use self::area_overlay_handler::AreaOverlayHandler;

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

mod bonus_text_arg_handler;

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

mod item_callback_handler;

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

mod root_view;
pub use self::root_view::RootView;

mod script_menu;
pub use self::script_menu::ScriptMenu;

mod trigger_activator;

mod window_fade;
pub use self::window_fade::WindowFade;

mod world_map_window;
pub use self::world_map_window::WorldMapWindow;

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::io::{MainLoopUpdater};
use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::widgets::{Button, ConfirmationWindow, Label};
use sulis_state::{ChangeListener, GameState};

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
            trigger_activator::activate(root, &cb.on_trigger, &cb.parent, &cb.target);
        }
    }

    fn is_exit(&self) -> bool {
        self.view.borrow().next_step.is_some()
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

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        GameState::add_party_listener(ChangeListener::invalidate("portrait_pane", &widget));

        let mut children = Vec::new();

        let selected = GameState::selected();
        for entity in GameState::party() {
            if !entity.borrow().show_portrait() {
                continue;
            }

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

pub struct UIBlocker {
    remaining_millis: u32,
}

impl UIBlocker {
    pub fn new(remaining_millis: u32) -> Rc<RefCell<UIBlocker>> {
        Rc::new(RefCell::new(UIBlocker { remaining_millis }))
    }
}

impl WidgetKind for UIBlocker {
    widget_kind!("ui_blocker");

    fn update(&mut self, widget: &Rc<RefCell<Widget>>, millis: u32) {
        if millis > self.remaining_millis {
            widget.borrow_mut().mark_for_removal();
        } else {
            self.remaining_millis -= millis;
        }
    }
}
