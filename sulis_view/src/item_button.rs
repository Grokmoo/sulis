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

use sulis_rules::{bonus_list::{AttackBuilder, AttackKindBuilder}, BonusList};
use sulis_module::{item::{format_item_value, format_item_weight, Slot}};
use sulis_state::{EntityState, GameState, ItemState, inventory::has_proficiency};
use sulis_core::io::event;
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_widgets::{Label, TextArea};
use {ItemActionMenu, MerchantWindow, PropWindow, RootView};

enum Kind {
    Prop { index: usize },
    Merchant { id: String },
    Inventory { player: Rc<RefCell<EntityState>> },
    Equipped { player: Rc<RefCell<EntityState>> },
}

pub struct ItemButton {
    icon: String,
    quantity: u32,
    item_index: usize,
    kind: Kind,
    actions: Vec<(String, Callback)>,

    item_window: Option<Rc<RefCell<Widget>>>,
}

const ITEM_BUTTON_NAME: &str = "item_button";

impl ItemButton {
    pub fn inventory(player: &Rc<RefCell<EntityState>>, icon: String,
                     quantity: u32, item_index: usize) -> Rc<RefCell<ItemButton>> {
        let player = Rc::clone(player);
        ItemButton::new(icon, quantity, item_index, Kind::Inventory { player })
    }

    pub fn equipped(player: &Rc<RefCell<EntityState>>, icon: String,
                    item_index: usize) -> Rc<RefCell<ItemButton>> {
        let player = Rc::clone(player);
        ItemButton::new(icon, 1, item_index, Kind::Equipped { player })
    }

    pub fn prop(icon: String, quantity: u32, item_index: usize,
                prop_index: usize) -> Rc<RefCell<ItemButton>> {
        ItemButton::new(icon, quantity, item_index, Kind::Prop { index: prop_index })
    }

    pub fn merchant(icon: String, quantity: u32, item_index: usize,
                    merchant_id: &str) -> Rc<RefCell<ItemButton>> {
        ItemButton::new(icon, quantity, item_index, Kind::Merchant { id: merchant_id.to_string() })
    }

    fn new(icon: String, quantity: u32, item_index: usize, kind: Kind) -> Rc<RefCell<ItemButton>> {
        Rc::new(RefCell::new(ItemButton {
            icon,
            quantity,
            item_index,
            kind,
            actions: Vec::new(),
            item_window: None,
        }))
    }

    pub fn add_action(&mut self, name: &str, cb: Callback) {
        self.actions.push((name.to_string(), cb));
    }

    fn remove_item_window(&mut self) {
        if self.item_window.is_some() {
            self.item_window.as_ref().unwrap().borrow_mut().mark_for_removal();
            self.item_window = None;
        }
    }

    fn get_item_state(&self) -> Option<ItemState> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        match self.kind {
            Kind::Equipped { ref player } | Kind::Inventory { ref player } => {
                let pc = player.borrow();
                match pc.actor.inventory().items.get(self.item_index) {
                    None => None,
                    Some(&(_, ref item_state)) => Some(item_state.clone())
                }
            }, Kind::Prop { index } => {
                if !area_state.prop_index_valid(index) { return None; }

                match area_state.get_prop(index).items() {
                    None => None,
                    Some(ref items) => {
                        match items.get(self.item_index) {
                            None => None,
                            Some(&(_, ref item_state)) => Some(item_state.clone())
                        }
                    }
                }
            }, Kind::Merchant { ref id } => {
                let merchant = area_state.get_merchant(id);
                let merchant = match merchant {
                    None => return None,
                    Some(ref merchant) => merchant,
                };

                match merchant.items().get(self.item_index) {
                    None => None,
                    Some(&(_, ref item_state)) => Some(item_state.clone())
                }
            }
        }
    }

    fn check_sell_action(&self, widget: &Rc<RefCell<Widget>>) -> Option<Callback> {
        match self.kind {
            Kind::Inventory { .. } => (),
            _ => return None
        }

        // TODO this is a hack putting this here.  but, the state of the merchant
        // window may change after the owing inventory window is opened
        let root = Widget::get_root(widget);
        let root_view = Widget::downcast_kind_mut::<RootView>(&root);
        if let Some(window_widget) = root_view.get_merchant_window(&root) {
            let merchant_window = Widget::downcast_kind_mut::<MerchantWindow>(&window_widget);
            Some(sell_item_cb(merchant_window.player(), self.item_index))
        } else {
            None
        }
    }

    fn add_price_text_arg(&self, root: &Rc<RefCell<Widget>>, item_window: &mut Widget,
                          item_state: &ItemState) {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        match self.kind {
            Kind::Merchant { ref id } => {
                let merchant = area_state.get_merchant(id);
                if let Some(ref merchant) = merchant {
                    let value = merchant.get_buy_price(item_state);
                    item_window.state.add_text_arg("price", &format_item_value(value));
                }
            }, Kind::Inventory { .. } | Kind::Equipped { .. } => {
                let root_view = Widget::downcast_kind_mut::<RootView>(&root);
                let merch_window = match root_view.get_merchant_window(&root) {
                    None => return,
                    Some(window) => window,
                };
                let window = Widget::downcast_kind_mut::<MerchantWindow>(&merch_window);
                let merchant = area_state.get_merchant(window.merchant_id());
                if let Some(ref merchant) = merchant {
                    let value = merchant.get_sell_price(item_state);
                    item_window.state.add_text_arg("price", &format_item_value(value));
                }
            }, _ => (),
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
        icon.borrow_mut().state.add_text_arg("icon", &self.icon);

        vec![icon, qty_label]
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);

        if self.item_window.is_some() { return true; }

        let item_state = self.get_item_state();
        let item_state = match item_state {
            None => return true,
            Some(item_state) => item_state,
        };

        let root = Widget::get_root(widget);
        let item_window = Widget::with_theme(TextArea::empty(), "item_window");
        {
            let mut item_window = item_window.borrow_mut();
            item_window.state.disable();
            item_window.state.set_position(widget.borrow().state.inner_right(),
            widget.borrow().state.inner_top());

            match self.kind {
                Kind::Inventory { ref player } => {
                    if !has_proficiency(&item_state, &player.borrow().actor.stats) {
                        item_window.state.add_text_arg("prof_not_met", "true");
                    }
                }, Kind::Merchant { .. } => {
                    let player = GameState::selected();
                    if player.len() > 0 {
                        if !has_proficiency(&item_state, &player[0].borrow().actor.stats) {
                            item_window.state.add_text_arg("prof_not_met", "true");
                        }
                    }
                },
                _ => (),
            }

            item_window.state.add_text_arg("name", &item_state.item.name);
            item_window.state.add_text_arg("value", &format_item_value(item_state.item.value));
            item_window.state.add_text_arg("weight", &format_item_weight(item_state.item.weight));
            self.add_price_text_arg(&root, &mut item_window, &item_state);

            match item_state.item.equippable {
                None => (),
                Some(ref equippable) => {
                    if let Some(ref attack) = equippable.attack {
                        add_attack_text_args(attack, &mut item_window.state);
                    }
                    add_bonus_text_args(&equippable.bonuses, &mut item_window.state);
                },
            }
        }
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

        match kind {
            event::ClickKind::Left => {
                let action = match self.check_sell_action(widget) {
                    Some(action) => Some(action),
                    None => {
                        match self.actions.first() {
                            None => None,
                            Some(ref action) => Some(action.1.clone()),
                        }
                    }
                };

                if let Some(action) = action {
                    action.call(widget, self);
                }
            },
            event::ClickKind::Right => {
                let menu = ItemActionMenu::new();

                if let Some(action) = self.check_sell_action(widget) {
                    menu.borrow_mut().add_action("Sell", action);
                }

                for &(ref name, ref cb) in self.actions.iter() {
                    menu.borrow_mut().add_action(name, cb.clone());
                }
                let menu = Widget::with_defaults(menu);
                menu.borrow_mut().state.set_modal(true);
                menu.borrow_mut().state.modal_remove_on_click_outside = true;
                let root = Widget::get_root(widget);
                Widget::add_child_to(&root, menu);
            },
            _ => return false,
        }

        true
    }
}

pub fn take_item_cb(entity: &Rc<RefCell<EntityState>>, prop_index: usize, index: usize) -> Callback {
    let entity = Rc::clone(entity);
    Callback::with(Box::new(move || {
        entity.borrow_mut().actor.take(prop_index, index);
    }))
}

pub fn equip_item_cb(entity: &Rc<RefCell<EntityState>>, index: usize) -> Callback {
    let entity = Rc::clone(entity);
    Callback::with(Box::new(move || {
        entity.borrow_mut().actor.equip(index);
    }))
}

pub fn buy_item_cb(entity: &Rc<RefCell<EntityState>>, merchant_id: &str, index: usize) -> Callback {
    let entity = Rc::clone(entity);
    let merchant_id = merchant_id.to_string();
    Callback::with(Box::new(move || {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        let mut merchant = area_state.get_merchant_mut(&merchant_id);
        let merchant = match merchant {
            None => return,
            Some(ref mut merchant) => merchant,
        };

        let value = match merchant.items().get(index) {
            None => return,
            Some(&(_, ref item_state)) => merchant.get_buy_price(item_state),
        };

        if entity.borrow().actor.inventory().coins() < value {
            return;
        }

        if let Some(item_state) = merchant.remove(index) {
            entity.borrow_mut().actor.add_coins(-value);
            entity.borrow_mut().actor.add_item(item_state);
        }
    }))
}

pub fn sell_item_cb(entity: &Rc<RefCell<EntityState>>, index: usize) -> Callback {
    let entity = Rc::clone(entity);
    Callback::new(Rc::new(move |widget, _| {
        let root = Widget::get_root(widget);
        let root_view = Widget::downcast_kind_mut::<RootView>(&root);
        let merchant = match root_view.get_merchant_window(&root) {
            None => return,
            Some(ref window) => {
                let merchant_window = Widget::downcast_kind_mut::<MerchantWindow>(&window);
                merchant_window.merchant_id().to_string()
            }
        };

        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        let mut merchant = area_state.get_merchant_mut(&merchant);
        let merchant = match merchant {
            None => return,
            Some(ref mut merchant) => merchant,
        };

        let item_state = entity.borrow_mut().actor.remove_item(index);
        if let Some(item_state) = item_state {
            let value = merchant.get_sell_price(&item_state);
            entity.borrow_mut().actor.add_coins(value);
            merchant.add(item_state);
        }
    }))
}

pub fn drop_item_cb(entity: &Rc<RefCell<EntityState>>, index: usize) -> Callback {
    let entity = Rc::clone(entity);
    Callback::new(Rc::new(move |widget, _| {
        drop_item(widget, &entity, index);
    }))
}

fn drop_item(widget: &Rc<RefCell<Widget>>, entity: &Rc<RefCell<EntityState>>, index: usize) {
    let root = Widget::get_root(widget);
    let root_view = Widget::downcast_kind_mut::<RootView>(&root);
    match root_view.get_prop_window(&root) {
        None => drop_to_ground(entity, index),
        Some(ref window) => {
            let prop_window = Widget::downcast_kind_mut::<PropWindow>(&window);
            let prop_index = prop_window.prop_index();
            drop_to_prop(entity, index, prop_index);
        }
    }
}

fn drop_to_prop(entity: &Rc<RefCell<EntityState>>, index: usize, prop_index: usize) {
    let area_state = GameState::area_state();
    let mut area_state = area_state.borrow_mut();
    if !area_state.prop_index_valid(prop_index) { return; }

    let prop_state = area_state.get_prop_mut(prop_index);
    let item_state = entity.borrow_mut().actor.remove_item(index);

    if let Some(item_state) = item_state {
        prop_state.add_item(item_state);
    }
}

fn drop_to_ground(entity: &Rc<RefCell<EntityState>>, index: usize) {
    let p = entity.borrow().location.to_point();
    let area_state = GameState::area_state();
    let mut area_state = area_state.borrow_mut();

    area_state.check_create_prop_container_at(p.x, p.y);
    if let Some(ref mut prop) = area_state.prop_mut_at(p.x, p.y) {
        let item_state = entity.borrow_mut().actor.remove_item(index);

        if let Some(item_state) = item_state {
            prop.add_item(item_state);
        }
    }
}

pub fn unequip_and_drop_item_cb(entity: &Rc<RefCell<EntityState>>, slot: Slot) -> Callback {
    let entity = Rc::clone(entity);
    Callback::new(Rc::new(move |widget, _| {
        let index = match entity.borrow_mut().actor.inventory().equipped.get(&slot) {
            None => return,
            Some(index) => *index,
        };

        entity.borrow_mut().actor.unequip(slot);
        drop_item(widget, &entity, index);
    }))
}

pub fn unequip_item_cb(entity: &Rc<RefCell<EntityState>>, slot: Slot) -> Callback {
    let entity = Rc::clone(entity);
    Callback::with(Box::new(move || {
        entity.borrow_mut().actor.unequip(slot);
    }))
}

pub fn add_attack_text_args(attack: &AttackBuilder, widget_state: &mut WidgetState) {
    widget_state.add_text_arg("min_damage", &attack.damage.min.to_string());
    widget_state.add_text_arg("max_damage", &attack.damage.max.to_string());
    if attack.damage.ap > 0 {
        widget_state.add_text_arg("armor_piercing", &attack.damage.ap.to_string());
    }
    add_if_present(widget_state, "damage_kind", attack.damage.kind);

    match attack.kind {
        AttackKindBuilder::Melee { reach } =>
            widget_state.add_text_arg("reach", &reach.to_string()),
            AttackKindBuilder::Ranged { range, .. } =>
                widget_state.add_text_arg("range", &range.to_string()),
    }

    let bonuses = &attack.bonuses;
    add_if_present(widget_state, "attack_crit_threshold", bonuses.crit_threshold);
    add_if_present(widget_state, "attack_hit_threshold", bonuses.hit_threshold);
    add_if_present(widget_state, "attack_graze_threshold", bonuses.graze_threshold);
    add_if_present(widget_state, "attack_graze_multiplier", bonuses.graze_multiplier);
    add_if_present(widget_state, "attack_crit_multiplier", bonuses.crit_multiplier);
    add_if_present(widget_state, "attack_accuracy", bonuses.accuracy);

    if let Some(ref damage) = bonuses.bonus_damage {
        widget_state.add_text_arg("attack_min_bonus_damage", &damage.min.to_string());
        widget_state.add_text_arg("attack_max_bonus_damage", &damage.max.to_string());
        if let Some(kind) = damage.kind {
            widget_state.add_text_arg("attack_bonus_damage_kind", &kind.to_string());
        }
    }
}

pub fn add_bonus_text_args(bonuses: &BonusList, widget_state: &mut WidgetState) {
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
        for (attr, value) in attributes.iter() {
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
    add_if_present(widget_state, "concealment", bonuses.concealment);
    add_if_present(widget_state, "crit_threshold", bonuses.crit_threshold);
    add_if_present(widget_state, "hit_threshold", bonuses.hit_threshold);
    add_if_present(widget_state, "graze_threshold", bonuses.graze_threshold);
    add_if_present(widget_state, "graze_multiplier", bonuses.graze_multiplier);
    add_if_present(widget_state, "crit_multiplier", bonuses.crit_multiplier);
}

fn add_if_present<T: Display>(widget_state: &mut WidgetState, text: &str, val: Option<T>) {
    if let Some(val) = val {
        widget_state.add_text_arg(text, &val.to_string());
    }
}
