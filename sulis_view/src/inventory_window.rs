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
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util;
use sulis_core::widgets::{Button, Label};
use sulis_module::{QuickSlot, Slot};
use sulis_state::{script::ScriptItemKind, ChangeListener, EntityState, GameState};

use crate::{item_callback_handler::*, item_list_pane::Filter, ItemButton, ItemListPane};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    entity: Rc<RefCell<EntityState>>,
    filter: Rc<Cell<Filter>>,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<InventoryWindow>> {
        Rc::new(RefCell::new(InventoryWindow {
            entity: Rc::clone(entity),
            filter: Rc::new(Cell::new(Filter::All)),
        }))
    }
}

impl WidgetKind for InventoryWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self, _widget: &Rc<RefCell<Widget>>) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed inventory window.");
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let start_time = time::Instant::now();

        let stash = GameState::party_stash();
        stash
            .borrow_mut()
            .listeners
            .add(ChangeListener::invalidate(NAME, widget));
        self.entity
            .borrow_mut()
            .actor
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
                let window = Widget::kind_mut::<InventoryWindow>(&widget_ref);
                window.entity = Rc::clone(entity);
                widget_ref.borrow_mut().invalidate_children();
            }),
        ));

        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<InventoryWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let swap_weapons = Widget::with_theme(Button::empty(), "swap_weapons");
        let entity_ref = Rc::clone(&self.entity);
        swap_weapons
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |_, _| {
                EntityState::swap_weapon_set(&entity_ref);
            })));
        swap_weapons
            .borrow_mut()
            .state
            .set_enabled(self.entity.borrow().actor.can_swap_weapons());

        let actor = &self.entity.borrow().actor;

        let item_list_pane =
            Widget::with_defaults(ItemListPane::new_entity(&self.entity, &self.filter));

        let equipped_area = Widget::empty("equipped_area");
        for slot in Slot::iter() {
            let theme_id = format!("{slot:?}_button").to_lowercase();

            match actor.inventory().equipped(*slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }
                Some(item_state) => {
                    let button = ItemButton::equipped(&self.entity, item_state, *slot);
                    if actor.can_unequip(*slot) {
                        let mut but = button.borrow_mut();
                        but.add_action("Unequip", unequip_item_cb(&self.entity, *slot), true);
                        if !item_state.item.quest {
                            but.add_action(
                                "Drop",
                                unequip_and_drop_item_cb(&self.entity, *slot),
                                false,
                            );
                        }
                    }

                    Widget::add_child_to(&equipped_area, Widget::with_theme(button, &theme_id));
                }
            }
        }
        Widget::add_child_to(&equipped_area, swap_weapons);

        for quick_slot in QuickSlot::iter() {
            let theme_id = format!("{quick_slot:?}_button").to_lowercase();

            match actor.inventory().quick(*quick_slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }
                Some(item_state) => {
                    let quantity = 1 + stash.borrow().items().get_quantity(item_state);
                    let but = ItemButton::quick(&self.entity, quantity, item_state, *quick_slot);

                    if actor.can_use_quick(*quick_slot) {
                        let kind = ScriptItemKind::Quick(*quick_slot);
                        but.borrow_mut()
                            .add_action("Use", use_item_cb(&self.entity, kind), true);
                    }

                    but.borrow_mut().add_action(
                        "Clear Slot",
                        clear_quickslot_cb(&self.entity, *quick_slot),
                        false,
                    );

                    Widget::add_child_to(&equipped_area, Widget::with_theme(but, &theme_id));
                }
            }
        }

        let stash_title = Widget::with_theme(Label::empty(), "stash_title");

        trace!(
            "Inventory window creation time: {}",
            util::format_elapsed_secs(start_time.elapsed())
        );

        vec![close, equipped_area, item_list_pane, stash_title]
    }
}
