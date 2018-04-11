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

use std::fmt::Display;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_rules::BonusList;
use sulis_module::item::Slot;
use sulis_state::{AreaState, EntityState, ChangeListener, GameState, ItemState};
use sulis_core::io::event;
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_widgets::{Button, Label, TextArea};
use sulis_rules::bonus_list::AttackKindBuilder;

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
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

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

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let ref actor = self.entity.borrow().actor;

        let list_content = Widget::empty("items_list");
        for (index, &(quantity, ref item)) in actor.inventory().items.iter().enumerate() {
            let mut quantity = quantity - actor.inventory().equipped_quantity(index);
            if quantity == 0 { continue; }

            let button = Widget::with_defaults(
                ItemButton::new(Some(item.item.icon.id()), quantity, Some(index), None, None));

            match item.item.equippable {
                Some(_) => {
                    button.borrow_mut().state.add_callback(Callback::with(Box::new(move || {
                        let pc = GameState::pc();
                        let mut pc = pc.borrow_mut();

                        pc.actor.equip(index);
                    })));
                },
                None => (),
            };

            Widget::add_child_to(&list_content, button);
        }

        let equipped_area = Widget::empty("equipped_area");
        for slot in Slot::iter() {
            let (enabled, icon) = match actor.inventory().get(*slot) {
                None => (false, None),
                Some(ref item_state) => (true, Some(item_state.item.icon.id())),
            };

            let theme_id = format!("{:?}_button", slot).to_lowercase();
            let item_button = ItemButton::new(icon, 1, actor.inventory().get_index(*slot), None, None);
            let button = Widget::with_theme(item_button.clone(), &theme_id);
            button.borrow_mut().state.set_enabled(enabled);

            button.borrow_mut().state.add_callback(Callback::with(Box::new(move || {
                let pc = GameState::pc();
                let mut pc = pc.borrow_mut();

                pc.actor.unequip(*slot);
            })));
            Widget::add_child_to(&equipped_area, button);
        }

        vec![title, close, equipped_area, list_content]
    }
}

pub struct ItemButton {
    icon: Option<String>,
    quantity: u32,
    button: Rc<RefCell<Button>>,
    item_window: Option<Rc<RefCell<Widget>>>,
    item_index: Option<usize>,
    prop_index: Option<usize>,
    merchant_id: Option<String>,
}

const ITEM_BUTTON_NAME: &str = "item_button";

impl ItemButton {
    pub fn new(icon: Option<String>, quantity: u32, index: Option<usize>,
               prop_index: Option<usize>, merchant_id: Option<String>) -> Rc<RefCell<ItemButton>> {
        Rc::new(RefCell::new(ItemButton {
            icon,
            quantity,
            button: Button::empty(),
            item_window: None,
            item_index: index,
            prop_index,
            merchant_id,
        }))
    }

    fn remove_item_window(&mut self) {
        if self.item_window.is_some() {
            self.item_window.as_ref().unwrap().borrow_mut().mark_for_removal();
            self.item_window = None;
        }
    }

    fn get_item_state<'a>(&self, area_state: &'a AreaState, pc: &'a EntityState) -> Option<&'a ItemState> {
        let item_index = match self.item_index {
            None => return None,
            Some(index) => index,
        };

        if let Some(ref merchant_id) = self.merchant_id {
            let merchant = area_state.get_merchant(merchant_id);
            let merchant = match merchant {
                None => return None,
                Some(ref merchant) => merchant,
            };

            if item_index >= merchant.items().len() { return None; }

            return Some(&merchant.items()[item_index].1);
        }

        match self.prop_index {
            None => {
                if item_index >= pc.actor.inventory().items.len() {
                    return None;
                }
                Some(&pc.actor.inventory().items[item_index].1)
            }, Some(prop_index) => {
                if !area_state.prop_index_valid(prop_index) { return None; }

                if item_index >= area_state.get_prop(prop_index).items().len() {
                    return None;
                }
                Some(&area_state.get_prop(prop_index).items()[item_index].1)
            }
        }
    }
}

impl WidgetKind for ItemButton {
    widget_kind!(ITEM_BUTTON_NAME);

    fn on_remove(&mut self) {
        self.remove_item_window();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let qty_label = Widget::with_theme(Label::empty(), "quantity_label");
        if self.quantity > 1 {
            qty_label.borrow_mut().state.add_text_arg("quantity", &self.quantity.to_string());
        }
        let icon = Widget::empty("icon");
        if let Some(ref val) = self.icon {
            icon.borrow_mut().state.add_text_arg("icon", &val);
        }

        vec![icon, qty_label]
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);

        if self.item_window.is_some() { return true; }

        let pc = GameState::pc();
        let pc = pc.borrow();
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let item_state = self.get_item_state(&*area_state, &*pc);
        let item_state = match item_state {
            None => return true,
            Some(ref item_state) => item_state,
        };

        let item_window = Widget::with_theme(TextArea::empty(), "item_window");
        {
            let mut item_window = item_window.borrow_mut();
            item_window.state.disable();
            item_window.state.set_position(widget.borrow().state.inner_right(),
            widget.borrow().state.inner_top());

            item_window.state.add_text_arg("name", &item_state.item.name);

            match item_state.item.equippable {
                None => (),
                Some(ref equippable) => {
                    add_bonus_text_args(&equippable.bonuses, &mut item_window.state);
                },
            }
        }
        let root = Widget::get_root(widget);
        Widget::add_child_to(&root, Rc::clone(&item_window));
        self.item_window = Some(item_window);

        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);

        self.remove_item_window();
        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.remove_item_window();
        self.button.borrow_mut().on_mouse_release(widget, kind)
    }
}

pub fn add_bonus_text_args(bonuses: &BonusList, widget_state: &mut WidgetState) {
    if let Some(ref attack) = bonuses.attack {
        widget_state.add_text_arg("min_damage", &attack.damage.min.to_string());
        widget_state.add_text_arg("max_damage", &attack.damage.max.to_string());
        add_if_present(widget_state, "damage_kind", attack.damage.kind);

        match attack.kind {
            AttackKindBuilder::Melee { reach } =>
                widget_state.add_text_arg("reach", &reach.to_string()),
            AttackKindBuilder::Ranged { range, .. } =>
                widget_state.add_text_arg("range", &range.to_string()),
        }
    }

    if let Some(ref damage) = bonuses.bonus_damage {
        widget_state.add_text_arg("min_bonus_damage", &damage.min.to_string());
        widget_state.add_text_arg("max_bonus_damage", &damage.max.to_string());
        if let Some(kind) = damage.kind {
            widget_state.add_text_arg("bonus_damage_kind", &kind.to_string());
        }
    }

    let mut armor_arg_added = false;
    if let Some(ref base_armor) = bonuses.base_armor {
        widget_state.add_text_arg("armor", &base_armor.to_string());
        armor_arg_added = true;
    }

    if let Some(ref armor_kinds) = bonuses.armor_kinds {
        if !armor_arg_added {
            widget_state.add_text_arg("armor", "0");
        }

        for (kind, amount) in armor_kinds.iter() {
            widget_state.add_text_arg(&format!("armor_{}", kind).to_lowercase(),
                                               &amount.to_string());
        }
    }

    if let Some(ref attributes) = bonuses.attributes {
        for &(attr, value) in attributes.iter() {
            widget_state.add_text_arg(&attr.short_name(), &value.to_string());
        }
    }

    add_if_present(widget_state, "bonus_ap", bonuses.ap);
    add_if_present(widget_state, "bonus_reach", bonuses.bonus_reach);
    add_if_present(widget_state, "bonus_range", bonuses.bonus_range);
    add_if_present(widget_state, "initiative", bonuses.initiative);
    add_if_present(widget_state, "hit_points", bonuses.hit_points);
    add_if_present(widget_state, "accuracy", bonuses.accuracy);
    add_if_present(widget_state, "defense", bonuses.defense);
    add_if_present(widget_state, "fortitude", bonuses.fortitude);
    add_if_present(widget_state, "reflex", bonuses.reflex);
    add_if_present(widget_state, "will", bonuses.will);
}

fn add_if_present<T: Display>(widget_state: &mut WidgetState, text: &str, val: Option<T>) {
    if let Some(val) = val {
        widget_state.add_text_arg(text, &val.to_string());
    }
}
