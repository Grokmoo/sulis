use module::{item, Actor};
use state::{ChangeListenerList, EntityState, Inventory};
use rules::{AttributeList, Damage, StatList};

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct ActorState {
    pub actor: Rc<Actor>,
    inventory: Rc<RefCell<Inventory>>,
    pub attributes: AttributeList,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
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
            listeners: ChangeListenerList::default(),
            hp: 0,
        }
    }

    pub fn can_attack(&self, target: &Rc<RefCell<EntityState>>) -> bool {
        true
    }

    pub fn attack(&mut self, target: &Rc<RefCell<EntityState>>) {
        let amount = self.stats.damage.roll();
        info!("'{}' attacks '{}' for {} damage", self.actor.name,
              target.borrow().actor.actor.name, amount);
        target.borrow_mut().remove_hp(amount);
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

    pub fn remove_hp(&mut self, hp: u32) {
        if hp > self.hp {
            self.hp = 0;
        } else {
            self.hp -= hp;
        }

        self.listeners.notify(&self);
    }

    pub fn init(&mut self) {
        self.hp = self.stats.max_hp;
    }

    pub fn compute_stats(&mut self) {
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

        self.listeners.notify(&self);
    }
}
