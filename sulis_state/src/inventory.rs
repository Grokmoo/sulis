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

use std::io::Error;
use std::slice::Iter;
use std::rc::Rc;
use std::collections::HashMap;

use sulis_core::image::Image;
use sulis_core::util::invalid_data_error;
use sulis_rules::{StatList, Slot, ItemKind, WeaponStyle, bonus::AttackKindBuilder, QuickSlot};
use sulis_module::{Actor, ImageLayer, Module};
use {ItemList, ItemState};
use save_state::ItemListEntrySaveState;

#[derive(Clone)]
pub struct Inventory {
    pub items: ItemList,
    pub equipped: HashMap<Slot, usize>,
    pub quick: HashMap<QuickSlot, usize>,
    pub coins: i32,
}

impl Inventory {
    pub fn new(actor: &Rc<Actor>) -> Inventory {
        let mut inv = Inventory::empty();
        for item in actor.items.iter() {
            inv.items.add(ItemState::new(Rc::clone(item)));
        }

        inv.coins = actor.coins;
        trace!("Populated initial inventory with {} items", inv.items.len());
        inv
    }

    pub fn empty() -> Inventory {
        Inventory {
            items: ItemList::new(),
            equipped: HashMap::new(),
            quick: HashMap::new(),
            coins: 0,
        }
    }

    pub fn load(&mut self, items: Vec<ItemListEntrySaveState>,
                equipped: Vec<Option<usize>>, quick: Vec<Option<usize>>) -> Result<(), Error> {
        for item_save in items {
           let item = match Module::item(&item_save.item.id) {
               None => invalid_data_error(&format!("No item with ID '{}'",
                                                   item_save.item.id)),
                Some(item) => Ok(item),
           }?;

           self.items.add_quantity(item_save.quantity, ItemState::new(item));
        }

        for (slot_index, slot) in Slot::iter().enumerate() {
            let slot = *slot;

            if slot_index > equipped.len() { break; }
            let index = match equipped[slot_index] {
                None => continue,
                Some(index) => index,
            };

            let item_state = match self.items.get(index) {
                None => invalid_data_error(&format!("Invalid equipped index {}", index)),
                Some((_, ref item_state)) => Ok(item_state),
            }?;

            let equippable = match item_state.item.equippable {
                None => invalid_data_error(&format!("Item at index '{}' is not equippable",
                                                    index)),
                Some(ref equip) => Ok(equip),
            }?;

            if equippable.slot != slot {
                let ok = match equippable.alternate_slot {
                    None => false,
                    Some(alt_slot) => alt_slot == slot,
                };

                if !ok {
                    return invalid_data_error(&format!("item at index '{}' invalid equip type",
                                                       index));
                }
            }

            self.equipped.insert(slot, index);
        }

        for (quick_index, quick_slot) in QuickSlot::iter().enumerate() {
            let quick_slot = *quick_slot;

            if quick_index > quick.len() { break; }
            let index = match quick[quick_index] {
                None => continue,
                Some(index) => index,
            };

            self.quick.insert(quick_slot, index);
        }

        Ok(())
    }

    pub fn weapon_style(&self) -> WeaponStyle {
        match self.get(Slot::HeldOff) {
            None => (),
            Some(ref item_state) => {
                match item_state.item.kind {
                    ItemKind::Armor { .. } => return WeaponStyle::Shielded,
                    ItemKind::Weapon { .. } => {
                        match self.get(Slot::HeldMain) {
                            None => return WeaponStyle::Single,
                            Some(_) => return WeaponStyle::DualWielding,
                        }
                    }
                    ItemKind::Other => (),
                }
            }
        }

        match self.get(Slot::HeldMain) {
            None => return WeaponStyle::Single,
            Some(ref item_state) => {
                match &item_state.item.equippable {
                    Some(ref equippable) => {
                        match equippable.attack {
                            Some(ref attack_builder) => {
                                if let AttackKindBuilder::Ranged { .. } = attack_builder.kind {
                                    return WeaponStyle::Ranged;
                                }
                            },
                            _ => (),
                        }

                        match equippable.blocks_slot {
                            Some(Slot::HeldOff) => return WeaponStyle::TwoHanded,
                            _ => (),
                        }

                        WeaponStyle::Single
                    },
                    None => unreachable!(),
                }
            }
        }
    }

    pub fn quick(&self, slot: &QuickSlot) -> Option<usize> {
        self.quick.get(slot).map(|i| *i)
    }

    pub fn equipped(&self, slot: &Slot) -> Option<usize> {
        self.equipped.get(slot).map(|i| *i)
    }

    pub fn coins(&self) -> i32 {
        self.coins
    }

    pub fn add_coins(&mut self, amount: i32) {
        self.coins += amount;
    }

    /// Removes one copy of the item at the specified index.  Will not
    /// remove an equipped item.
    pub fn remove(&mut self, index: usize) -> Option<ItemState> {
        let quantity = match self.items.get(index) {
            None => return None,
            Some(&(qty, _)) => qty,
        };

        if self.equipped_quantity(index) == quantity {
            return None;
        }

        let mut quick_refs = Vec::new();
        for(slot, quick_index) in self.quick.iter_mut() {
            if quantity == 1 && *quick_index > index {
                *quick_index -= 1;
            } else if *quick_index == index {
                quick_refs.push(*slot);
            }
        }

        if quantity == 1 {
            for (_, equipped_index) in self.equipped.iter_mut() {
                if *equipped_index > index {
                    *equipped_index -= 1;
                }
            }
        }

        let quantity = quantity - 1;
        if quick_refs.len() as u32 > quantity {
            let to_remove = quick_refs.len() - quantity as usize;
            for i in 0..to_remove {
                self.quick.remove(&quick_refs[i]);
            }
        }

        self.items.remove(index)
    }

    pub fn get_quick_index(&self, quick_slot: QuickSlot) -> Option<usize> {
        match self.quick.get(&quick_slot) {
            None => None,
            Some(index) => Some(*index),
        }
    }

    pub fn get_quick(&self, quick_slot: QuickSlot) -> Option<&ItemState> {
        let index = match self.quick.get(&quick_slot) {
            None => return None,
            Some(index) => *index,
        };

        Some(&self.items[index].1)
    }

    pub fn get_index(&self, slot: Slot) -> Option<usize> {
        match self.equipped.get(&slot) {
            None => None,
            Some(index) => Some(*index),
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&ItemState> {
        let index = match self.equipped.get(&slot) {
            None => return None,
            Some(index) => *index,
        };

        Some(&self.items[index].1)
    }

    /// Returns an iterator traversing all equipped items
    /// in this inventory.  This will only include slots that actually
    /// have an item equipped
    pub fn equipped_iter(&self) -> EquippedIterator {
        EquippedIterator {
            slot_iterator: Slot::iter(),
            inventory: &self,
        }
    }

    /// Returns the number of times the item at the index is equipped.
    /// This can be more than 1 if the item has a quantity greater than 1
    /// and is equipped to both slot and alt slot
    pub fn equipped_quantity(&self, index: usize) -> u32 {
        let (slot, alt_slot) = match self.items.get(index) {
            None => return 0,
            Some(&(_, ref item)) => match &item.item.equippable {
                &None => return 0,
                &Some(ref equippable) => (equippable.slot, equippable.alternate_slot),
            }
        };

        let mut count = 0;
        if self.equipped.get(&slot) == Some(&index) { count += 1; }
        if let Some(alt_slot) = alt_slot {
            if self.equipped.get(&alt_slot) == Some(&index) { count += 1; }
        }

        count
    }

    /// checks whether the item at the given index is equipped.
    /// returns true if it is, false otherwise
    pub fn is_equipped(&self, index: usize) -> bool {
        self.equipped_quantity(index) > 0
    }

    pub fn swap_weapon_set(&mut self) {
        let cur_main = self.get_index(Slot::HeldMain);
        let cur_off = self.get_index(Slot::HeldOff);

        let cur_alt_main = self.get_quick_index(QuickSlot::AltHeldMain);
        let cur_alt_off = self.get_quick_index(QuickSlot::AltHeldOff);

        match cur_main {
            None => self.quick.remove(&QuickSlot::AltHeldMain),
            Some(index) => self.quick.insert(QuickSlot::AltHeldMain, index),
        };

        match cur_off {
            None => self.quick.remove(&QuickSlot::AltHeldOff),
            Some(index) => self.quick.insert(QuickSlot::AltHeldOff, index),
        };

        match cur_alt_main {
            None => self.equipped.remove(&Slot::HeldMain),
            Some(index) => self.equipped.insert(Slot::HeldMain, index),
        };

        match cur_alt_off {
            None => self.equipped.remove(&Slot::HeldOff),
            Some(index) => self.equipped.insert(Slot::HeldOff, index),
        };
    }

    pub fn clear_quick(&mut self, quick_slot: QuickSlot) {
        self.quick.remove(&quick_slot);
    }

    /// attempts to set the item at the given index as a quick item.  returns
    /// true if successful, false if not
    pub fn set_quick(&mut self, index: usize, quick_slot: QuickSlot) -> bool {
        {
            let item_state = match self.items.get(index) {
                None => return false,
                Some(&(_, ref item)) => item,
            };

            if item_state.item.usable.is_none() { return false; }
        }

        self.quick.insert(quick_slot, index);
        true
    }

    /// equips the item at the given index.  returns true if the item
    /// was equipped.  false if the item does not exist
    pub fn equip(&mut self, index: usize, stats: &StatList) -> bool {
        trace!("Attempting equip of item at '{}", index);

        let (slot, alt_slot, blocked_slot) = match self.items.get(index) {
            None => return false,
            Some(&(_, ref item)) => {
                if !has_proficiency(item, stats) { return false; }

                match &item.item.equippable {
                    &None => return false,
                    &Some(ref equippable) => {
                        trace!("Found matching slot '{:?}'", equippable.slot);

                        (equippable.slot, equippable.alternate_slot, equippable.blocks_slot)
                    }
                }
            }
        };

        // determine blocking slots that will need to be removed for primary and
        // alt, depending on which one we pick
        let mut to_remove_primary = None;
        let mut to_remove_alt = None;
        for item in self.equipped_iter() {
            if let Some(ref equippable) = item.item.equippable {
                if let Some(ref blocked_slot) = equippable.blocks_slot {
                    if *blocked_slot == slot {
                        to_remove_primary = Some(equippable.slot);
                    }

                    if alt_slot.is_some() && *blocked_slot == alt_slot.unwrap() {
                        to_remove_alt = Some(equippable.slot);
                    }
                }
            }
        }

        // now determine whether to go with primary or alt based on how many
        // items need to be removed from each.  prefer primary in a tie
        let slot_to_use = if alt_slot.is_none() {
            slot
        } else {
            let alt_slot = alt_slot.unwrap();
            let mut alt_count = 0;
            let mut primary_count = 0;
            if to_remove_primary.is_some() { primary_count += 1; }
            if self.equipped.get(&slot).is_some() { primary_count += 1; }
            if to_remove_alt.is_some() { alt_count += 1; }
            if self.equipped.get(&alt_slot).is_some() { alt_count += 1; }

            if alt_count < primary_count {
                alt_slot
            } else {
                slot
            }
        };

        if let Some(slot) = to_remove_primary {
            if !self.unequip(slot) { return false; }
        }

        if !self.unequip(slot_to_use) { return false; }

        if let Some(blocked_slot) = blocked_slot {
            if !self.unequip(blocked_slot) { return false; }
        }

        debug!("Equipping item at '{}' into '{:?}'", index, slot_to_use);
        self.equipped.insert(slot_to_use, index);

        true
    }

    pub fn clear_quickslot(&mut self, quick_slot: QuickSlot) -> bool {
        self.quick.remove(&quick_slot);
        true
    }

    /// unequips the item in the specified slot.  returns true if the
    /// slot is empty, or the item is able to be unequipped.  false if
    /// the item cannot be unequipped.
    pub fn unequip(&mut self, slot: Slot) -> bool {
        self.equipped.remove(&slot);

        true
    }

    pub fn get_image_layers(&self) -> HashMap<ImageLayer, Rc<Image>> {
        let mut layers = HashMap::new();

        for (slot, index) in self.equipped.iter() {
            let item_state = &self.items[*index].1;

            let equippable = match item_state.item.equippable {
                None => continue,
                Some(ref equippable) => equippable,
            };

            let iter = if equippable.slot == *slot {
                // the item is in it's primary slot
                match item_state.item.image_iter() {
                    None => continue,
                    Some(iter) => iter,
                }
            } else {
                // the item must be in it's secondary slot
                match item_state.item.alt_image_iter() {
                    None => continue,
                    Some(iter) => iter,
                }
            };

            for (layer, ref image) in iter {
                layers.insert(*layer, Rc::clone(image));
            }
        }

        layers
    }
}

pub struct EquippedIterator<'a> {
    slot_iterator: Iter<'static, Slot>,
    inventory: &'a Inventory,
}

impl<'a> Iterator for EquippedIterator<'a> {
    type Item = &'a ItemState;
    fn next(&mut self) -> Option<&'a ItemState> {
        loop {
            let slot = match self.slot_iterator.next() {
                None => return None,
                Some(slot) => slot,
            };

            let index = match self.inventory.equipped.get(&slot) {
                None => continue,
                Some(index) => *index,
            };

            return Some(&self.inventory.items[index].1);
        }
    }
}

pub fn has_proficiency(item: &ItemState, stats: &StatList) -> bool {
    match item.item.kind {
        ItemKind::Other => true,
        ItemKind::Armor { kind } => stats.has_armor_proficiency(kind),
        ItemKind::Weapon { kind } => stats.has_weapon_proficiency(kind),
    }
}
