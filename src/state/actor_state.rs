use module::{item, Actor};
use state::{ChangeListener, Inventory};
use rules::{AttributeList, Damage, StatList};

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct ActorState {
    pub actor: Rc<Actor>,
    inventory: Rc<RefCell<Inventory>>,
    pub attributes: AttributeList,
    pub stats: StatList,
    change_listeners: Vec<ChangeListener<ActorState>>,
    hp: u32,
}

impl PartialEq for ActorState {
    fn eq(&self, other: &ActorState) -> bool {
        Rc::ptr_eq(&self.actor, &other.actor)
    }
}

impl ActorState {
    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let inventory = Rc::new(RefCell::new(Inventory::new(&actor)));
        ActorState {
            actor,
            inventory,
            attributes: AttributeList::default(),
            stats: StatList::default(),
            change_listeners: Vec::new(),
            hp: 0,
        }
    }

    /// Removes all change listeners from this state with the given ID
    pub fn remove_change_listeners(&mut self, id: &'static str) {
        self.change_listeners.retain(|listener| listener.id() != id);
    }

    pub fn add_change_listener(&mut self, listener: ChangeListener<ActorState>) {
        self.change_listeners.push(listener);
    }

    pub fn equip(&mut self, index: usize) -> bool {
        let result = self.inventory.borrow_mut().equip(index);
        self.compute_stats();

        result
    }

    pub fn unequip(&mut self, slot: item::Slot) -> bool {
        let result = self.inventory.borrow_mut().unequip(slot);
        self.compute_stats();

        result
    }

    pub fn inventory(&self) -> Ref<Inventory> {
        self.inventory.borrow()
    }

    pub fn hp(&self) -> u32 {
        self.hp
    }

    pub fn init(&mut self) {
        self.hp = self.stats.max_hp;
    }

    pub fn compute_stats(&mut self) {
        for listener in self.change_listeners.iter() {
            listener.call(&self);
        }

        let mut max_damage = Damage::default();

        for item_state in self.inventory.borrow().equipped_iter() {
            let equippable = match item_state.item.equippable {
                None => continue,
                Some(equippable) => equippable,
            };

            if let Some(damage) = equippable.damage {
                max_damage = Damage::max(damage, max_damage);
            }
        }

        self.stats.damage = max_damage;

        let mut max_hp: u32 = 0;
        for &(ref class, level) in self.actor.levels.iter() {
            max_hp += class.hp_per_level * level;
        }
        self.stats.max_hp = max_hp;
    }
}
