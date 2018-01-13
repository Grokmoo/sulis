use std::rc::Rc;
use std::collections::HashMap;

use module::Actor;
use module::item::Slot;
use state::ItemState;

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

    /// checks whether the item at the given index is equipped.
    /// returns true if it is, false otherwise
    pub fn is_equipped(&self, index: usize) -> bool {
        let slot = match self.items.get(index) {
            None => return false,
            Some(item) => match &item.item.slot {
                &None => return false,
                &Some(slot) => slot,
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
            Some(item) => match &item.item.slot {
                &None => return false,
                &Some(slot) => slot,
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
}
