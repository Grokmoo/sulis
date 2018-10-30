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

use std::time;
use std::any::Any;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

use sulis_core::util;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use sulis_rules::{Slot, QuickSlot};
use sulis_state::{EntityState, ChangeListener, GameState, script::ScriptItemKind};

use {item_button::*, ItemListPane, ItemButton, item_list_pane::Filter};

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

    fn on_remove(&mut self) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed inventory window.");
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let start_time = time::Instant::now();

        let stash = GameState::party_stash();
        stash.borrow_mut().listeners.add(ChangeListener::invalidate(NAME, widget));
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let combat_active = GameState::is_combat_active();

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |entity| {
            let entity = match entity {
                None => return,
                Some(entity) => entity,
            };
            let window = Widget::downcast_kind_mut::<InventoryWindow>(&widget_ref);
            window.entity = Rc::clone(entity);
            widget_ref.borrow_mut().invalidate_children();
        })));

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let ref actor = self.entity.borrow().actor;

        let item_list_pane = Widget::with_defaults(ItemListPane::new_entity(&self.entity, &self.filter));

        let equipped_area = Widget::empty("equipped_area");
        for slot in Slot::iter() {
            let theme_id = format!("{:?}_button", slot).to_lowercase();

            match actor.inventory().equipped(*slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }, Some(item_state) => {
                    let button = ItemButton::equipped(&self.entity, &item_state.item, *slot);
                    if !combat_active {
                        let mut but = button.borrow_mut();
                        but.add_action("Unequip", unequip_item_cb(&self.entity, *slot));
                        but.add_action("Drop", unequip_and_drop_item_cb(&self.entity, *slot));
                    }

                    Widget::add_child_to(&equipped_area, Widget::with_theme(button, &theme_id));
                }
            }
        }

        for quick_slot in QuickSlot::iter() {
            let theme_id = format!("{:?}_button", quick_slot).to_lowercase();

            match actor.inventory().quick(*quick_slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }, Some(item_state) => {
                    let quantity = 1 + stash.borrow().items().get_quantity(&item_state);
                    let but = ItemButton::quick(&self.entity, quantity, &item_state.item,
                        *quick_slot);

                    if actor.can_use_quick(*quick_slot) {
                        let kind = ScriptItemKind::Quick(*quick_slot);
                        but.borrow_mut().add_action("Use", use_item_cb(&self.entity, kind));
                    }

                    but.borrow_mut().add_action("Clear Slot",
                                                clear_quickslot_cb(&self.entity, *quick_slot));

                    Widget::add_child_to(&equipped_area, Widget::with_theme(but, &theme_id));
                }
            }
        }


        let stash_title = Widget::with_theme(Label::empty(), "stash_title");

        trace!("Inventory window creation time: {}",
               util::format_elapsed_secs(start_time.elapsed()));

        vec![title, close, equipped_area, item_list_pane, stash_title]
    }
}
