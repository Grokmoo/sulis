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
        let slot = match self.items.get(index) {
            None => return false,
            Some(item) => match &item.item.equippable {
                &None => return false,
                &Some(equippable) => equippable.slot,
            }
        };

        self.equipped.get(&slot) == Some(&index)
    }

    /// equips the item at the given index.  returns true if the item
    /// was equipped.  false if the item does not exist
    pub fn equip(&mut self, index: usize) -> bool {
        trace!("Attempting equip of item at '{}", index);
        let slot = match self.items.get(index) {
            None => return false,
            Some(item) => match &item.item.equippable {
                &None => return false,
                &Some(equippable) => equippable.slot,
            }
        };
        trace!("Found matching slot '{:?}'", slot);

        if !self.unequip(slot) {
            return false;
        }

        debug!("Equipping item at '{}' into '{:?}'", index, slot);
        self.equipped.insert(slot, index);

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

            let layer = slot_to_layer(*slot);
            match item_state.item.image_for_layer(layer) {
                None => continue,
                Some(ref image) => layers.insert(layer, Rc::clone(image)),
            };
        }

        layers
    }
}

fn slot_to_layer(slot: Slot) -> ImageLayer {
    use sulis_module::item::Slot::*;
    match slot {
        Head => ImageLayer::Head,
        Torso => ImageLayer::Torso,
        Hands => ImageLayer::Hands,
        HeldMain => ImageLayer::HeldMain,
        HeldOff => ImageLayer::HeldOff,
        Legs => ImageLayer::Legs,
        Feet => ImageLayer::Feet,
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
