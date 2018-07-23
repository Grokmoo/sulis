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
use std::cell::RefCell;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, ScrollPane};
use sulis_rules::{Slot, QuickSlot};
use sulis_module::{Module};
use sulis_state::{EntityState, ChangeListener, GameState, inventory::has_proficiency};

use {item_button::*, ItemButton};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    entity: Rc<RefCell<EntityState>>,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<InventoryWindow>> {
        Rc::new(RefCell::new(InventoryWindow {
            entity: Rc::clone(entity)
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
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

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

        let scrollpane = ScrollPane::new();
        let list_content = Widget::with_theme(scrollpane.clone(), "items_list");
        for (index, &(quantity, ref item)) in actor.inventory().items.iter().enumerate() {
            let mut quantity = quantity - actor.inventory().equipped_quantity(index);
            if quantity == 0 { continue; }

            let item_but = ItemButton::inventory(&self.entity, item.item.icon.id(),
                quantity, index);

            if let Some(_) = item.item.usable {
                let mut but = item_but.borrow_mut();
                but.add_action("Set Quickslot 1",
                               set_quickslot_cb(&self.entity, index, QuickSlot::Usable1));
                but.add_action("Set Quickslot 2",
                               set_quickslot_cb(&self.entity, index, QuickSlot::Usable2));
                but.add_action("Set Quickslot 3",
                               set_quickslot_cb(&self.entity, index, QuickSlot::Usable3));
                but.add_action("Set Quickslot 4",
                               set_quickslot_cb(&self.entity, index, QuickSlot::Usable4));
            }

            if let Some(_) = item.item.equippable {
                if has_proficiency(&item, &actor.stats) {
                    item_but.borrow_mut().add_action("Equip", equip_item_cb(&self.entity, index));
                }
            }
            item_but.borrow_mut().add_action("Drop", drop_item_cb(&self.entity, index));

            scrollpane.borrow().add_to_content(Widget::with_defaults(item_but));
        }

        let equipped_area = Widget::empty("equipped_area");
        for slot in Slot::iter() {
            let theme_id = format!("{:?}_button", slot).to_lowercase();

            match actor.inventory().get_index(*slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }, Some(index) => {
                    let item_state = actor.inventory().get(*slot).unwrap();

                    let button = ItemButton::equipped(&self.entity, item_state.item.icon.id(),
                        index);
                    button.borrow_mut().add_action("Unequip", unequip_item_cb(&self.entity, *slot));
                    button.borrow_mut().add_action("Drop",
                                                   unequip_and_drop_item_cb(&self.entity, *slot));

                    Widget::add_child_to(&equipped_area, Widget::with_theme(button, &theme_id));
                }
            }
        }

        for quick_slot in QuickSlot::iter() {
            let theme_id = format!("{:?}_button", quick_slot).to_lowercase();

            match actor.inventory().get_quick_index(*quick_slot) {
                None => {
                    let button = Widget::empty(&theme_id);
                    button.borrow_mut().state.set_enabled(false);
                    Widget::add_child_to(&equipped_area, button);
                }, Some(index) => {
                    let (qty, item_state) = actor.inventory().items.get(index).unwrap();

                    let button = ItemButton::inventory(&self.entity, item_state.item.icon.id(),
                                                      *qty, index);
                    Widget::add_child_to(&equipped_area, Widget::with_theme(button, &theme_id));
                }
            }
        }

        let coins_item = match Module::item(&Module::rules().coins_item) {
            None => {
                warn!("Unable to find coins item");
                return Vec::new();
            }, Some(item) => item,
        };
        let amount = actor.inventory().coins() as f32 / Module::rules().item_value_display_factor;
        let button = ItemButton::inventory(&self.entity, coins_item.icon.id(), amount as u32, 0);
        let coins_button = Widget::with_theme(button, "coins_button");
        coins_button.borrow_mut().state.set_enabled(false);

        let stash_title = Widget::with_theme(Label::empty(), "stash_title");
        vec![title, close, equipped_area, list_content, coins_button, stash_title]
    }
}
