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

use std::slice::Iter;
use std::rc::Rc;
use std::collections::HashMap;

use sulis_core::image::Image;
use sulis_module::{Actor, ImageLayer};
use sulis_module::item::Slot;
use ItemState;

#[derive(Clone)]
pub struct Inventory {
    pub items: Vec<ItemState>,
    pub equipped: HashMap<Slot, usize>,
}

impl Inventory {
    pub fn new(actor: &Rc<Actor>) -> Inventory {
        let mut items: Vec<ItemState> = Vec::new();

        for item in actor.items.iter() {
            items.push(ItemState::new(Rc::clone(item)));
        }

        trace!("Populated initial inventory with {} items", items.len());
        Inventory {
            items,
            equipped: HashMap::new(),
        }
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

        self.items.get(index)
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

    /// checks whether the item at the given index is equipped.
    /// returns true if it is, false otherwise
    pub fn is_equipped(&self, index: usize) -> bool {
        let (slot, alt_slot) = match self.items.get(index) {
            None => return false,
            Some(item) => match &item.item.equippable {
                &None => return false,
                &Some(ref equippable) => (equippable.slot, equippable.alternate_slot),
            }
        };

        if self.equipped.get(&slot) == Some(&index) { return true; }

        if let Some(alt_slot) = alt_slot {
            if self.equipped.get(&alt_slot) == Some(&index) { return true; }
        }

        false
    }

    /// equips the item at the given index.  returns true if the item
    /// was equipped.  false if the item does not exist
    pub fn equip(&mut self, index: usize) -> bool {
        trace!("Attempting equip of item at '{}", index);

        let (slot, alt_slot, blocked_slot) = match self.items.get(index) {
            None => return false,
            Some(item) => match &item.item.equippable {
                &None => return false,
                &Some(ref equippable) => {
                    trace!("Found matching slot '{:?}'", equippable.slot);

                    (equippable.slot, equippable.alternate_slot, equippable.blocks_slot)
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
            let item_state = &self.items[*index];

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

            return Some(&self.inventory.items[index]);
        }
    }
}
