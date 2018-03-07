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

use sulis_core::io::GraphicsRenderer;
use sulis_core::image::{LayeredImage};
use sulis_core::ui::{color, Color};
use sulis_module::{item, Actor, Module};
use {ChangeListenerList, EntityState, GameState, Inventory};
use sulis_rules::{HitKind, StatList};

pub struct ActorState {
    pub actor: Rc<Actor>,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
    hp: i32,
    ap: u32,
    xp: u32,
    has_level_up: bool,
    inventory: Inventory,
    image: LayeredImage,
}

impl ActorState {
    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let mut inventory = Inventory::new(&actor);
        for index in actor.to_equip.iter() {
            inventory.equip(*index);
        }

        let image = LayeredImage::new(actor.image_layers().get_list(actor.sex,
                                                                    actor.hair_color,
                                                                    actor.skin_color), actor.hue);
        let attrs = actor.attributes;

        let xp = actor.xp;
        ActorState {
            actor,
            inventory,
            stats: StatList::new(attrs),
            listeners: ChangeListenerList::default(),
            hp: 0,
            ap: 0,
            xp,
            has_level_up: false,
            image,
        }
    }

    pub fn level_up(&mut self, new_actor: Actor) {
        self.actor = Rc::new(new_actor);
        self.compute_stats();
        self.init();
    }

    pub fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                              x: f32, y: f32, millis: u32) {
        self.image.draw(renderer, scale_x, scale_y, x, y, millis);
    }

    pub fn can_reach(&self, dist: f32) -> bool {
        dist < self.stats.attack_distance()
    }

    pub(crate) fn can_attack(&self, _target: &Rc<RefCell<EntityState>>, dist: f32) -> bool {
        trace!("Checking can attack for '{}'.  Distance to target is {}",
               self.actor.name, dist);

        let attack_ap = Module::rules().attack_ap;
        if self.ap < attack_ap { return false; }

        self.can_reach(dist)
    }

    pub fn attack(&mut self, target: &Rc<RefCell<EntityState>>) -> (String, Color) {
        self.remove_ap(Module::rules().attack_ap);
        info!("'{}' attacks '{}'", self.actor.name, target.borrow().actor.actor.name);
        let rules = Module::rules();

        let mut color = color::GRAY;
        let mut damage_str = String::new();
        let mut not_first = false;

        for ref attack in self.stats.attacks.iter() {
            if not_first { damage_str.push_str(", "); }

            let accuracy = self.stats.accuracy;
            let defense = target.borrow().actor.stats.defense;
            let hit_kind = rules.attack_roll(accuracy, defense);

            let damage_multiplier = match hit_kind {
                HitKind::Miss => {
                    debug!("Miss");
                    damage_str.push_str("Miss");
                    not_first = true;
                    continue;
                },
                HitKind::Graze => rules.graze_damage_multiplier,
                HitKind::Hit => rules.hit_damage_multiplier,
                HitKind::Crit => rules.crit_damage_multiplier,
            };

            let damage = attack.roll_damage(&target.borrow().actor.stats.armor, damage_multiplier);

            debug!("{:?}. {:?} damage", hit_kind, damage);

            if !damage.is_empty() {
                color = color::RED;
                let mut total = 0;
                for (_kind, amount) in damage {
                    total += amount;
                }

                target.borrow_mut().remove_hp(total);
                damage_str.push_str(&format!("{:?}: {}", hit_kind, total));
            }

            not_first = true;
        }

        self.check_death(target);
        (damage_str, color)
    }

    pub fn take_all(&mut self, prop_index: usize) {
        let area_state = GameState::area_state();
        let prop_state = &mut area_state.borrow_mut().props[prop_index];

        if prop_state.num_items() > 0 {
            let mut i = prop_state.num_items() - 1;
            loop {
                let item_state = prop_state.remove_item(i);
                self.inventory.add(item_state);

                if i == 0 { break; }

                i -= 1;
            }
            self.listeners.notify(&self);
        }
    }

    pub fn take(&mut self, prop_index: usize, item_index: usize) {
        let area_state = GameState::area_state();
        let prop_state = &mut area_state.borrow_mut().props[prop_index];

        let item_state = prop_state.remove_item(item_index);
        self.inventory.add(item_state);
        self.listeners.notify(&self);
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

    pub fn check_death(&mut self, target: &Rc<RefCell<EntityState>>) {
        if target.borrow().actor.hp() > 0 { return; }

        let target = target.borrow();
        let reward = match target.actor.actor.reward {
            None => return,
            Some(ref reward) => reward,
        };

        self.add_xp(reward.xp);
    }

    pub fn has_level_up(&self) -> bool {
        self.has_level_up
    }

    pub fn add_xp(&mut self, xp: u32) {
        self.xp += xp;
        self.compute_stats();
    }

    pub fn xp(&self) -> u32 {
        self.xp
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
        self.stats = StatList::new(self.actor.attributes);

        let layers = self.actor.image_layers().get_list_with(self.actor.sex, &self.actor.race,
                                                             self.actor.hair_color, self.actor.skin_color,
                                                             self.inventory.get_image_layers());
        self.image = LayeredImage::new(layers, self.actor.hue);

        let rules = Module::rules();
        self.stats.initiative = rules.base_initiative;
        self.stats.add(&self.actor.race.base_stats);

        for &(ref class, level) in self.actor.levels.iter() {
            self.stats.add_multiple(&class.bonuses_per_level, level);
        }

        let mut attacks_list = Vec::new();
        for ref item_state in self.inventory.equipped_iter() {
            let equippable = match item_state.item.equippable {
                None => continue,
                Some(ref equippable) => {
                    if let Some(ref attack) = equippable.bonuses.attack {
                        attacks_list.push(attack);
                    }

                    equippable
                }
            };

            self.stats.add(&equippable.bonuses);
        }

        let multiplier = if attacks_list.is_empty() {
            if let Some(ref attack) = self.actor.race.base_stats.attack {
                attacks_list.push(attack);
            }

            1.0
        } else if attacks_list.len() > 1 {
            rules.dual_wield_damage_multiplier
        } else {
            1.0
        };

        self.stats.finalize(attacks_list, multiplier, rules.base_attribute);

        self.has_level_up = rules.get_xp_for_next_level(self.actor.total_level) < self.xp;

        self.listeners.notify(&self);
    }
}
