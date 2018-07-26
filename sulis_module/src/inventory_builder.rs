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

use std::rc::Rc;
use std::collections::HashMap;

use sulis_rules::{Slot, QuickSlot};
use {Item, Module};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct InventoryBuilder {
    equipped: HashMap<Slot, String>,
    quick: HashMap<QuickSlot, String>,
    items: Vec<String>,
    coins: u32,
}

fn equippable_to(item: &Rc<Item>, item_id: &str, slot: Slot) -> bool {
    match &item.equippable {
        None => {
            warn!("Unequippable item '{}' found in equip slot for inventory", item_id);
            return false;
        },
        Some(ref equippable) => {
            if equippable.slot != slot {
                let alt = equippable.alternate_slot;
                if alt.is_none() || alt.unwrap() != slot {
                    warn!("Item '{}' not equippable to slot {:?} in inventory",
                          item_id, slot);
                    return false;
                }
            }
        }
    }
    true
}

impl InventoryBuilder {
    /// Returns the number of coins specified for this inventory
    pub fn coins(&self) -> u32 {
        self.coins
    }

    /// Iterates over the items in this inventory, validating that they exist
    pub fn item_iter<'a>(&'a self) -> impl Iterator<Item=Rc<Item>> + 'a {
        self.items.iter().filter_map(|item_id| {
            match Module::item(item_id) {
                None => {
                    warn!("Item with id '{}' not found for inventory item", item_id);
                    None
                }, Some(item) => Some(item),
            }
        })
    }

    /// Provides an iterator over all the items in this inventory.
    /// Validates as much as possible that items are valid for the specified slots,
    /// but cannot do validations that depend on the actor.
    pub fn equipped_iter<'a>(&'a self) -> impl Iterator<Item=(Slot, Rc<Item>)> + 'a {
        self.equipped.iter().filter_map(|(slot, item_id)| {
            let slot = *slot;
            let item = match Module::item(item_id) {
                None => {
                    warn!("Item with id '{}' not found in equipped for inventory", item_id);
                    return None;
                }, Some(item) => item,
            };

            if !equippable_to(&item, item_id, slot) { return None; }

            Some((slot, item))
        })
    }

    /// Provides an iterator over all the quick slot item in this defined inventory.
    /// Validates as much as possible that the items are valid for the slots, but cannot do
    /// any validation that also depends on the actor.
    pub fn quick_iter<'a>(&'a self) -> impl Iterator<Item=(QuickSlot, Rc<Item>)> + 'a {
        self.quick.iter().filter_map(|(slot, item_id)| {
            let slot = *slot;
            let item = match Module::item(item_id) {
                None => {
                    warn!("Item with id '{}' not found in quick for inventory", item_id);
                    return None;
                }, Some(item) => item,
            };

            use sulis_rules::QuickSlot::*;
            match slot {
                AltHeldMain => {
                    if !equippable_to(&item, item_id, Slot::HeldMain) { return None; }
                },
                AltHeldOff => {
                    if !equippable_to(&item, item_id, Slot::HeldOff) { return None; }
                },
                Usable1 | Usable2 | Usable3 | Usable4 => {
                    if item.usable.is_none() { return None; }
                }
            }

            Some((slot, item))
        })
    }
}

impl Default for InventoryBuilder {
    fn default() -> InventoryBuilder {
        InventoryBuilder {
            equipped: HashMap::new(),
            quick: HashMap::new(),
            items: Vec::new(),
            coins: 0,
        }
    }
}
