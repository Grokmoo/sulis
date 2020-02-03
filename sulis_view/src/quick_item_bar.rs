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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    item_callback_handler::{clear_quickslot_cb, use_item_cb},
    ItemButton,
};
use sulis_core::io::{keyboard_event::Key, InputAction};
use sulis_core::ui::{animation_state, Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label};
use sulis_module::QuickSlot;
use sulis_state::{script::ScriptItemKind, ChangeListener, EntityState, GameState};

pub const NAME: &str = "quick_item_bar";

pub struct QuickItemBar {
    entity: Rc<RefCell<EntityState>>,
    swap_weapons_key: Option<Key>,
    quick_item_keys: [Option<Key>; 4],
    quick_items: Vec<Option<Rc<RefCell<Widget>>>>,
}

impl QuickItemBar {
    pub fn new(
        entity: &Rc<RefCell<EntityState>>,
        keybindings: &HashMap<InputAction, Key>,
    ) -> Rc<RefCell<QuickItemBar>> {
        let swap_weapons_key = keybindings.get(&InputAction::SwapWeapons).cloned();
        let quick_item_keys = [
            keybindings.get(&InputAction::QuickItem1).cloned(),
            keybindings.get(&InputAction::QuickItem2).cloned(),
            keybindings.get(&InputAction::QuickItem3).cloned(),
            keybindings.get(&InputAction::QuickItem4).cloned(),
        ];

        Rc::new(RefCell::new(QuickItemBar {
            entity: Rc::clone(entity),
            swap_weapons_key,
            quick_item_keys,
            quick_items: Vec::new(),
        }))
    }

    pub fn check_handle_keybinding(&self, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            SwapWeapons => self.swap(),
            QuickItem1 => self.do_quick_item(0),
            QuickItem2 => self.do_quick_item(1),
            QuickItem3 => self.do_quick_item(2),
            QuickItem4 => self.do_quick_item(3),
            _ => return false,
        }

        true
    }

    fn do_quick_item(&self, index: usize) {
        if self.quick_items.len() <= index {
            return;
        }

        let widget = match &self.quick_items[index] {
            None => return,
            Some(widget) => widget,
        };

        let button: &mut ItemButton = Widget::kind_mut(widget);
        button.fire_left_click_action(widget);
    }

    fn swap(&self) {
        EntityState::swap_weapon_set(&self.entity);
    }

    fn add_button(
        &mut self,
        slot: QuickSlot,
        key_index: usize,
        theme_id: &str,
    ) -> Rc<RefCell<Widget>> {
        let key = self.quick_item_keys[key_index];
        let (button, add) = create_button(&self.entity, slot, key, theme_id);

        if add {
            self.quick_items.push(Some(Rc::clone(&button)));
        } else {
            self.quick_items.push(None);
        }
        button
    }
}

impl WidgetKind for QuickItemBar {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.quick_items.clear();

        {
            let mut entity = self.entity.borrow_mut();
            entity
                .actor
                .listeners
                .add(ChangeListener::invalidate(NAME, widget));
        }

        let stash = GameState::party_stash();
        stash
            .borrow_mut()
            .listeners
            .add(ChangeListener::invalidate(NAME, widget));

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(
            NAME,
            Box::new(move |entity| {
                let entity = match entity {
                    None => return,
                    Some(entity) => entity,
                };
                let bar = Widget::kind_mut::<QuickItemBar>(&widget_ref);
                bar.entity = Rc::clone(entity);
                widget_ref.borrow_mut().invalidate_children();
            }),
        ));

        let title = Widget::with_theme(Label::empty(), "title");

        let swap_weapons = Widget::with_theme(Button::empty(), "swap_weapons");
        {
            let state = &mut swap_weapons.borrow_mut().state;
            state.add_callback(Callback::new(Rc::new(|widget, _| {
                let (_, bar) = Widget::parent_mut::<QuickItemBar>(widget);
                bar.swap();
            })));
            state.set_enabled(self.entity.borrow().actor.can_swap_weapons());

            if let Some(key) = self.swap_weapons_key {
                state.add_text_arg("keybinding", &key.short_name());
            }
        }

        let usable1 = self.add_button(QuickSlot::Usable1, 0, "usable1");
        let usable2 = self.add_button(QuickSlot::Usable2, 1, "usable2");
        let usable3 = self.add_button(QuickSlot::Usable3, 2, "usable3");
        let usable4 = self.add_button(QuickSlot::Usable4, 3, "usable4");

        vec![title, swap_weapons, usable1, usable2, usable3, usable4]
    }
}

fn create_button(
    entity: &Rc<RefCell<EntityState>>,
    slot: QuickSlot,
    key: Option<Key>,
    theme_id: &str,
) -> (Rc<RefCell<Widget>>, bool) {
    let stash = GameState::party_stash();
    let actor = &entity.borrow().actor;
    match actor.inventory().quick(slot) {
        None => {
            let button = Widget::empty(theme_id);
            button.borrow_mut().state.set_enabled(false);
            (button, false)
        }
        Some(item_state) => {
            let quantity = 1 + stash.borrow().items().get_quantity(&item_state);
            let kind = ScriptItemKind::Quick(slot);
            let button = ItemButton::quick(entity, quantity, &item_state, slot);
            button.borrow_mut().set_keyboard_shortcut(key);
            button
                .borrow_mut()
                .add_action("Use", use_item_cb(entity, kind), true);
            button
                .borrow_mut()
                .add_action("Clear Slot", clear_quickslot_cb(entity, slot), false);
            let widget = Widget::with_theme(button, theme_id);
            if actor.can_use_quick(slot) {
                widget
                    .borrow_mut()
                    .state
                    .animation_state
                    .remove(animation_state::Kind::Custom1);
            } else {
                widget
                    .borrow_mut()
                    .state
                    .animation_state
                    .add(animation_state::Kind::Custom1);
            }
            (widget, true)
        }
    }
}
