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
use std::cell::{RefCell};

use sulis_core::image::{LayeredImage};
use sulis_core::io::DrawList;
use sulis_module::{item, Actor, Module};
use {ChangeListenerList, EntityState, Inventory};
use sulis_rules::{AttributeList, StatList};

pub struct ActorState {
    pub actor: Rc<Actor>,
    pub attributes: AttributeList,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
    hp: i32,
    ap: u32,
    inventory: Inventory,
    image: LayeredImage,
}

impl PartialEq for ActorState {
    fn eq(&self, other: &ActorState) -> bool {
        Rc::ptr_eq(&self.actor, &other.actor)
    }
}

impl ActorState {
    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let mut inventory = Inventory::new(&actor);
        for index in actor.to_equip.iter() {
            inventory.equip(*index);
        }

        let image = LayeredImage::new(actor.image_layers().get_list(actor.sex));

        ActorState {
            actor,
            inventory,
            attributes: AttributeList::default(),
            stats: StatList::default(),
            listeners: ChangeListenerList::default(),
            hp: 0,
            ap: 0,
            image,
        }
    }

    pub fn append_to_draw_list(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        self.image.append_to(draw_list, x, y, millis);
        self.actor.check_add_swap_hue(draw_list);
    }

    pub fn can_reach(&self, dist: f32) -> bool {
        dist < self.stats.reach
    }

    pub(crate) fn can_attack(&self, _target: &Rc<RefCell<EntityState>>, dist: f32) -> bool {
        trace!("Checking can attack for '{}' with reach of {}.  Distance to target is {}",
               self.actor.name, self.stats.reach, dist);

        let attack_ap = Module::rules().attack_ap;
        if self.ap < attack_ap { return false; }

        self.can_reach(dist)
    }

    pub fn attack(&mut self, target: &Rc<RefCell<EntityState>>) {
        let amount = self.stats.damage.roll();
        let armor = target.borrow().actor.stats.armor.amount(self.stats.damage.kind);
        info!("'{}' attacks '{}' for {} damage vs {} armor", self.actor.name,
              target.borrow().actor.actor.name, amount, armor);
        if amount > armor {
            target.borrow_mut().remove_hp(amount - armor);
        }
        self.remove_ap(Module::rules().attack_ap);
    }

    pub fn equip(&mut self, index: usize) -> bool {
        let result = self.inventory.equip(index);
        self.compute_stats();

        result
    }

    pub fn unequip(&mut self, slot: item::Slot) -> bool {
        let result = self.inventory.unequip(slot);
        self.compute_stats();

        result
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn hp(&self) -> i32 {
        self.hp
    }

    pub fn ap(&self) -> u32 {
        self.ap
    }

    pub fn get_move_ap_cost(&self, squares: u32) -> u32 {
        let rules = Module::rules();
        rules.movement_ap * squares
    }

    pub(crate) fn remove_ap(&mut self, ap: u32) {
        if ap > self.ap {
            self.ap = 0;
        } else {
            self.ap -= ap;
        }

        self.listeners.notify(&self);
    }

    pub(crate) fn remove_hp(&mut self, hp: u32) {
        if hp as i32 > self.hp {
            self.hp = 0;
        } else {
            self.hp -= hp as i32;
        }

        self.listeners.notify(&self);
    }

    pub fn init(&mut self) {
        self.hp = self.stats.max_hp;
    }

    pub fn init_turn(&mut self) {
        let rules = Module::rules();

        self.ap = rules.base_ap;
        self.listeners.notify(&self);
    }

    pub fn end_turn(&mut self) {
        self.ap = 0;
    }

    pub fn compute_stats(&mut self) {
        debug!("Compute stats for '{}'", self.actor.name);
        self.stats = StatList::default();

        self.image = LayeredImage::new(self.actor.image_layers()
                                      .get_list_with(self.actor.sex, &self.actor.race,
                                                     self.inventory.get_image_layers()));

        let rules = Module::rules();
        self.stats.initiative = rules.base_initiative;
        for &(ref class, level) in self.actor.levels.iter() {
            self.stats.max_hp += (class.hp_per_level * level) as i32;
        }

        for ref item_state in self.inventory.equipped_iter() {
            let equippable = match item_state.item.equippable {
                None => continue,
                Some(ref equippable) => equippable,
            };

            self.stats.add(&equippable.bonuses);
        }

        self.listeners.notify(&self);
    }
}
