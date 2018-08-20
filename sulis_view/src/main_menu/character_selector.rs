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
use std::cell::{RefCell};

use sulis_core::ui::*;
use sulis_state::{ActorState, NextGameStep};
use sulis_module::{Actor, Module};
use sulis_widgets::{Button, ConfirmationWindow, Label, TextArea};

use character_window::create_details_text_box;
use {CharacterBuilder, LoadingScreen, main_menu::MainMenu};

pub struct CharacterSelector {
    selected: Option<Rc<Actor>>,
    first_add: bool,
    main_menu: Rc<RefCell<Widget>>,
    to_select: Option<String>,
}

impl CharacterSelector {
    pub fn new(main_menu: Rc<RefCell<Widget>>) -> Rc<RefCell<CharacterSelector>> {
        Rc::new(RefCell::new(CharacterSelector {
            selected: None,
            first_add: true,
            main_menu,
            to_select: None,
        }))
    }

    pub fn set_to_select(&mut self, actor_id: String) {
        self.to_select = Some(actor_id);
    }
}

impl WidgetKind for CharacterSelector {
    widget_kind!("character_selector");

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to main menu widget");

        let title = Widget::with_theme(Label::empty(), "title");
        let chars_title = Widget::with_theme(Label::empty(), "characters_title");

        let new_character_button = Widget::with_theme(Button::empty(), "new_character_button");
        let char_selector_widget = Rc::clone(widget);
        new_character_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let root = Widget::get_root(&widget);

            let builder = Widget::with_defaults(CharacterBuilder::new(&char_selector_widget));
            Widget::add_child_to(&root, builder);
        })));

        let delete_char_button = Widget::with_theme(Button::empty(), "delete_character_button");

        let (actor_name, actor_id) = match self.selected {
            None => (String::new(), String::new()),
            Some(ref actor) => (actor.name.to_string(), actor.id.to_string()),
        };
        delete_char_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let root = Widget::get_root(&widget);

            let actor_id = actor_id.clone();
            let parent = Widget::get_parent(&widget);
            let window = ConfirmationWindow::new(Callback::new(Rc::new(move |widget, _| {
                Module::delete_character(&actor_id);
                let widget_parent = Widget::get_parent(&widget);
                widget_parent.borrow_mut().mark_for_removal();

                let selector = Widget::downcast_kind_mut::<CharacterSelector>(&parent);
                selector.selected = None;
                parent.borrow_mut().invalidate_children();
            })));
            {
                let window = window.borrow();
                window.title().borrow_mut().state.add_text_arg("name", &actor_name);
            }
            let window_widget = Widget::with_theme(window, "delete_character_confirmation_window");
            window_widget.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, window_widget);
        })));

        let to_select = match self.to_select.take() {
            None => "".to_string(),
            Some(to_select) => to_select,
        };

        let mut must_create_character = true;
        let characters_pane = Widget::empty("characters_pane");
        {
            let characters = Module::get_available_characters();
            for actor in characters {
                let actor = Rc::new(actor);
                trace!("Adding button for {}", actor.id);

                if actor.id == to_select {
                    self.selected = Some(Rc::clone(&actor));
                }

                let actor_button = Widget::with_theme(Button::empty(), "character_button");
                actor_button.borrow_mut().state.add_text_arg("name", &actor.name);
                if let Some(ref portrait) = actor.portrait {
                    actor_button.borrow_mut().state.add_text_arg("portrait", &portrait.id());
                }

                if let Some(ref selected) = self.selected {
                    if actor.id == selected.id {
                        actor_button.borrow_mut().state.set_active(true);
                    }
                }

                actor_button.borrow_mut().state.add_callback(actor_callback(actor));

                must_create_character = false;
                Widget::add_child_to(&characters_pane, actor_button);
            }
        }

        let play_button = Widget::with_theme(Button::empty(), "play_button");
        play_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let selector = Widget::downcast_kind_mut::<CharacterSelector>(&parent);
            let selected = match selector.selected {
                None => return,
                Some(ref selected) => Rc::clone(selected),
            };

            let root = Widget::get_root(&widget);
            let window = Widget::downcast_kind_mut::<MainMenu>(&root);
            window.next_step = Some(NextGameStep::NewCampaign { pc_actor: selected });

            let loading_screen = Widget::with_defaults(LoadingScreen::new());
            loading_screen.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, loading_screen);
        })));

        let details = if let Some(ref actor) = self.selected {
            let mut actor_state = ActorState::new(Rc::clone(actor));
            actor_state.compute_stats();
            actor_state.init();
            actor_state.init_turn();
            create_details_text_box(&actor_state)
        } else {
            Widget::with_theme(TextArea::empty(), "details")
        };

        delete_char_button.borrow_mut().state.set_enabled(self.selected.is_some());
        play_button.borrow_mut().state.set_enabled(self.selected.is_some());

        if self.first_add && must_create_character {
            let menu = Widget::downcast_kind_mut::<MainMenu>(&self.main_menu);
            debug!("Showing character builder");
            menu.char_builder_to_add = Some(CharacterBuilder::new(widget));
        }
        self.first_add = false;

        vec![title, chars_title, characters_pane, new_character_button,
            delete_char_button, play_button, details]
    }
}

fn actor_callback(actor: Rc<Actor>) -> Callback {
    Callback::new(Rc::new(move |widget, _| {
        let parent = Widget::go_up_tree(&widget, 2);
        let selector = Widget::downcast_kind_mut::<CharacterSelector>(&parent);
        selector.selected = Some(Rc::clone(&actor));
        parent.borrow_mut().invalidate_children();
    }))
}
