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
use std::rc::Rc;

use crate::{
    item_button::{clear_quickslot_cb, use_item_cb},
    ItemButton,
};
use sulis_core::ui::{animation_state, Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label};
use sulis_module::QuickSlot;
use sulis_state::{script::ScriptItemKind, ChangeListener, EntityState, GameState};

pub const NAME: &str = "quick_item_bar";

pub struct QuickItemBar {
    entity: Rc<RefCell<EntityState>>,
}

impl QuickItemBar {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<QuickItemBar>> {
        Rc::new(RefCell::new(QuickItemBar {
            entity: Rc::clone(entity),
        }))
    }
}

impl WidgetKind for QuickItemBar {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
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
        swap_weapons
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (_, bar) = Widget::parent_mut::<QuickItemBar>(widget);
                bar.entity.borrow_mut().actor.swap_weapon_set();
            })));
        swap_weapons
            .borrow_mut()
            .state
            .set_enabled(self.entity.borrow().actor.can_swap_weapons());

        let usable1 = create_button(&self.entity, QuickSlot::Usable1, "usable1");
        let usable2 = create_button(&self.entity, QuickSlot::Usable2, "usable2");
        let usable3 = create_button(&self.entity, QuickSlot::Usable3, "usable3");
        let usable4 = create_button(&self.entity, QuickSlot::Usable4, "usable4");

        vec![title, swap_weapons, usable1, usable2, usable3, usable4]
    }
}

fn create_button(
    entity: &Rc<RefCell<EntityState>>,
    slot: QuickSlot,
    theme_id: &str,
) -> Rc<RefCell<Widget>> {
    let stash = GameState::party_stash();
    let actor = &entity.borrow().actor;
    match actor.inventory().quick(slot) {
        None => {
            let button = Widget::empty(theme_id);
            button.borrow_mut().state.set_enabled(false);
            button
        }
        Some(item_state) => {
            let quantity = 1 + stash.borrow().items().get_quantity(&item_state);
            let kind = ScriptItemKind::Quick(slot);
            let button = ItemButton::quick(entity, quantity, &item_state, slot);
            button
                .borrow_mut()
                .add_action("Use", use_item_cb(entity, kind), true);
            button
                .borrow_mut()
                .add_action("Clear Slot", clear_quickslot_cb(entity, slot), false);
            let widget = Widget::with_theme(button.clone(), theme_id);
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
            widget
        }
    }
}
