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
use std::slice::{Iter, IterMut};
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::HashMap;

use sulis_core::io::GraphicsRenderer;
use sulis_core::image::{LayeredImage};
use sulis_core::ui::{color, Color};
use sulis_core::util::invalid_data_error;
use sulis_module::{item, Actor, Module, ActorBuilder};
use sulis_rules::{Attack, AttackKind, HitKind, StatList};
use {AbilityState, ChangeListenerList, Effect, EntityState, GameState, Inventory, ItemState, ScriptCallback};
use save_state::ActorSaveState;

pub struct ActorState {
    pub actor: Rc<Actor>,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
    hp: i32,
    ap: u32,
    overflow_ap: i32,
    xp: u32,
    has_level_up: bool,
    inventory: Inventory,
    effects: Vec<Effect>,
    image: LayeredImage,
    pub(crate) ability_states: HashMap<String, AbilityState>,
    texture_cache_invalid: bool,
}

impl ActorState {
    pub fn load(save: ActorSaveState, base: Option<ActorBuilder>) -> Result<ActorState, Error> {
        let actor = match base {
            None => {
                match Module::actor(&save.id) {
                    None => invalid_data_error(&format!("No actor with id '{}'", save.id)),
                    Some(actor) => Ok(actor),
                }?
            }
            Some(builder) => {
                Rc::new(Module::load_actor(builder)?)
            }
        };

        let attrs = actor.attributes;

        let image = LayeredImage::new(actor.image_layers()
            .get_list(actor.sex, actor.hair_color, actor.skin_color), actor.hue);

        let mut ability_states = HashMap::new();
        for ability in actor.abilities.iter() {
            if ability.active.is_none() { continue; }

            let mut ability_state = AbilityState::new(ability);

            match save.ability_states.get(&ability.id) {
                None => (),
                Some(ref ability_save) => {
                    ability_state.remaining_duration = ability_save.remaining_duration;
                }
            }

            ability_states.insert(ability.id.to_string(), ability_state);
        }

        let mut inventory = Inventory::empty();
        inventory.load(save.items, save.equipped)?;
        inventory.add_coins(save.coins);

        Ok(ActorState {
            actor,
            inventory,
            stats: StatList::new(attrs),
            listeners: ChangeListenerList::default(),
            hp: save.hp,
            ap: save.ap,
            overflow_ap: save.overflow_ap,
            xp: save.xp,
            has_level_up: false,
            image,
            effects: Vec::new(),
            ability_states,
            texture_cache_invalid: false,
        })
    }

    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let inventory = Inventory::new(&actor);

        let image = LayeredImage::new(actor.image_layers().get_list(actor.sex,
                                                                    actor.hair_color,
                                                                    actor.skin_color), actor.hue);
        let attrs = actor.attributes;

        let mut ability_states = HashMap::new();
        for ability in actor.abilities.iter() {
            if ability.active.is_none() { continue; }

            ability_states.insert(ability.id.to_string(), AbilityState::new(ability));
        }

        let to_equip = actor.to_equip.clone();

        let xp = actor.xp;
        let mut actor_state = ActorState {
            actor,
            inventory,
            stats: StatList::new(attrs),
            listeners: ChangeListenerList::default(),
            hp: 0,
            ap: 0,
            overflow_ap: 0,
            xp,
            has_level_up: false,
            image,
            effects: Vec::new(),
            ability_states,
            texture_cache_invalid: false,
        };

        actor_state.compute_stats();
        for index in to_equip.iter() {
            actor_state.inventory.equip(*index, &actor_state.stats);
        }

        actor_state
    }

    pub fn check_texture_cache_invalid(&mut self) -> bool {
        if self.texture_cache_invalid {
            self.texture_cache_invalid = false;
            true
        } else {
            false
        }
    }

    pub fn ability_state(&mut self, id: &str) -> Option<&mut AbilityState> {
        self.ability_states.get_mut(id)
    }

    pub fn can_activate(&self, id: &str) -> bool {
        match self.ability_states.get(id) {
            None => false,
            Some(ref state) => {
                if self.ap < state.activate_ap() { return false; }

                state.is_available()
            }
        }
    }

    pub fn activate_ability_state(&mut self, id: &str) {
        match self.ability_states.get_mut(id) {
            None => (),
            Some(ref mut state) => state.activate(),
        }
    }

    pub fn effects_iter_mut<'a>(&'a mut self) -> IterMut<'a, Effect> {
        self.effects.iter_mut()
    }

    pub fn effects_iter<'a>(&'a self) -> Iter<'a, Effect> {
        self.effects.iter()
    }

    pub fn replace_actor(&mut self, new_actor: Actor) {
        self.actor = Rc::new(new_actor);

        for ability in self.actor.abilities.iter() {
            if ability.active.is_none() { continue; }

            self.ability_states.insert(ability.id.to_string(), AbilityState::new(ability));
        }

        self.compute_stats();
        self.init();
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                              x: f32, y: f32, millis: u32) {
        self.image.draw(renderer, scale_x, scale_y, x, y, millis);
    }

    pub fn draw_to_texture(&self, renderer: &mut GraphicsRenderer, texture_id: &str, scale_x: f32, scale_y: f32,
                              x: f32, y: f32) {
        self.image.draw_to_texture(renderer, texture_id, scale_x, scale_y, x, y);
    }

    pub fn can_reach(&self, dist: f32) -> bool {
        dist < self.stats.attack_distance()
    }

    pub(crate) fn can_weapon_attack(&self, _target: &Rc<RefCell<EntityState>>, dist: f32) -> bool {
        trace!("Checking can attack for '{}'.  Distance to target is {}",
               self.actor.name, dist);

        let attack_ap = Module::rules().attack_ap;
        if self.ap < attack_ap { return false; }

        self.can_reach(dist)
    }

    pub fn weapon_attack(parent: &Rc<RefCell<EntityState>>,
                         target: &Rc<RefCell<EntityState>>) -> (HitKind, String, Color) {
        if target.borrow_mut().actor.hp() <= 0 { return (HitKind::Miss, "Miss".to_string(), color::GRAY); }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name, target.borrow().actor.actor.name);

        let mut color = color::GRAY;
        let mut damage_str = String::new();
        let mut not_first = false;
        let mut hit_kind = HitKind::Miss;

        let attacks = parent.borrow().actor.stats.attacks.clone();
        for attack in attacks {
            if not_first { damage_str.push_str(", "); }

            let (hit, attack_result, attack_color) = ActorState::attack_internal(parent, target, &attack);
            if attack_color != color::GRAY {
                color = attack_color;
            }

            damage_str.push_str(&attack_result);

            if hit > hit_kind {
                hit_kind = hit;
            }

            not_first = true;
        }

        ActorState::check_death(parent, target);
        (hit_kind, damage_str, color)
    }

    pub fn attack(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                  attack: &Attack) -> (HitKind, String, Color) {
        if target.borrow_mut().actor.hp() <= 0 { return (HitKind::Miss, "Miss".to_string(), color::GRAY); }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name, target.borrow().actor.actor.name);

        let result = ActorState::attack_internal(parent, target, attack);

        ActorState::check_death(parent, target);
        result
    }

    fn attack_internal(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                       attack: &Attack) -> (HitKind, String, Color) {
        let rules = Module::rules();
        let accuracy = parent.borrow().actor.stats.accuracy;

        if !rules.concealment_roll(target.borrow().actor.stats.concealment) {
            debug!("Concealment miss");
            return (HitKind::Miss, "Concealment".to_string(), color::GRAY);
        }

        let defense = {
            let target_stats = &target.borrow().actor.stats;
            match attack.kind {
                AttackKind::Fortitude => target_stats.fortitude,
                AttackKind::Reflex => target_stats.reflex,
                AttackKind::Will => target_stats.will,
                AttackKind::Melee { .. } | AttackKind::Ranged { .. } => target_stats.defense,
            }
        };

        let (hit_kind, damage_multiplier) = {
            let parent_stats = &parent.borrow().actor.stats;
            let hit_kind = parent_stats.attack_roll(defense);
            let damage_multiplier = match hit_kind {
                HitKind::Miss => {
                    debug!("Miss");
                    return (HitKind::Miss, "Miss".to_string(), color::GRAY);
                },
                HitKind::Graze => parent_stats.graze_multiplier,
                HitKind::Hit => 1.0,
                HitKind::Crit => parent_stats.crit_multiplier,
            };
            (hit_kind, damage_multiplier)
        };

        debug!("Accuracy {} vs defense {}: {:?}", accuracy, defense, hit_kind);

        let damage = attack.roll_damage(&target.borrow().actor.stats.armor, damage_multiplier);

        debug!("{:?}. {:?} damage", hit_kind, damage);

        if !damage.is_empty() {
            let mut total = 0;
            for (_kind, amount) in damage {
                total += amount;
            }

            target.borrow_mut().remove_hp(total);
            return (hit_kind, format!("{:?}: {}", hit_kind, total), color::RED);
        } else if attack.damage.max() == 0 {
            // if attack cannot do any damage
            return (hit_kind, format!("{:?}", hit_kind), color::RED);
        } else {
            return (hit_kind, format!("{:?}: {}", hit_kind, 0), color::GRAY);
        }
    }

    fn check_add_coins(&mut self, quantity: u32, item_state: &ItemState) -> bool {
        let coins_id = &Module::rules().coins_item;

        if &item_state.item.id == coins_id {
            let qty = quantity as i32 * Module::rules().item_value_display_factor as i32;
            self.inventory.add_coins(qty);
            true
        } else {
            false
        }
    }

    pub fn take_all(&mut self, prop_index: usize) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        let prop_state = area_state.get_prop_mut(prop_index);

        let num_items = match prop_state.items() {
            None => return,
            Some(ref items) => items.len(),
        };

        if num_items > 0 {
            let mut i = num_items - 1;
            loop {
                if let Some((qty, item_state)) = prop_state.remove_all_at(i) {
                    if !self.check_add_coins(qty, &item_state) {
                        self.inventory.items.add_quantity(qty, item_state);
                    }
                }

                if i == 0 { break; }

                i -= 1;
            }
            self.listeners.notify(&self);
        }
    }

    pub fn take(&mut self, prop_index: usize, item_index: usize) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        let prop_state = area_state.get_prop_mut(prop_index);

        if let Some((qty, item_state)) = prop_state.remove_all_at(item_index) {
            if !self.check_add_coins(qty, &item_state) {
                self.inventory.items.add_quantity(qty, item_state);
            }
        }

        self.listeners.notify(&self);
    }

    pub fn add_item(&mut self, item_state: ItemState) {
        if !self.check_add_coins(1, &item_state) {
            self.inventory.items.add(item_state);
        }
        self.listeners.notify(&self);
    }

    pub fn equip(&mut self, index: usize) -> bool {
        let result = self.inventory.equip(index, &self.stats);
        self.compute_stats();
        self.texture_cache_invalid = true;

        result
    }

    pub fn unequip(&mut self, slot: item::Slot) -> bool {
        let result = self.inventory.unequip(slot);
        self.compute_stats();
        self.texture_cache_invalid = true;

        result
    }

    /// removes one item at the specified index from this actor's inventory.
    /// will not remove an equipped item
    pub fn remove_item(&mut self, index: usize) -> Option<ItemState> {
        let item = self.inventory.remove(index);

        if item.is_some() {
            self.compute_stats();
            // in case item was equipped
            self.texture_cache_invalid = true;
        }

        item
    }

    pub fn add_coins(&mut self, amount: i32) {
        self.inventory.add_coins(amount);
        self.listeners.notify(&self);
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn check_death(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {
        if target.borrow().actor.hp() > 0 { return; }

        let area_state = GameState::area_state();

        if let Some(index) = target.borrow().ai_group() {
            area_state.borrow().check_encounter_cleared(index, parent, target);
        }

        let reward = {
            let target = target.borrow();
            match target.actor.actor.reward {
                None => return,
                Some(ref reward) => reward.clone(),
            }
        };

        debug!("Adding XP {} to '{}'", reward.xp, parent.borrow().actor.actor.id);
        if parent.borrow().is_party_member() {
            for member in GameState::party().iter() {
                member.borrow_mut().add_xp(reward.xp);
            }
        } else {
            parent.borrow_mut().add_xp(reward.xp);
        }

        let loot = match reward.loot {
            None => return,
            Some(ref loot) => loot,
        };

        trace!("Checking for loot drop.");
        let items = loot.generate_with_chance(reward.loot_chance);
        if items.is_empty() { return; }

        trace!("Dropping loot with {} items", items.len());
        let p = target.borrow().location.to_point();
        let mut area_state = area_state.borrow_mut();

        area_state.check_create_prop_container_at(p.x, p.y);
        match area_state.prop_mut_at(p.x, p.y) {
            None => (),
            Some(ref mut prop) => {
                prop.add_items(items);
            }
        }
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

    pub fn overflow_ap(&self) -> i32 {
        self.overflow_ap
    }

    pub fn ap(&self) -> u32 {
        self.ap
    }

    pub fn get_move_ap_cost(&self, squares: u32) -> u32 {
        let rules = Module::rules();
        rules.movement_ap * squares
    }

    pub fn set_overflow_ap(&mut self, ap: i32) {
        let rules = Module::rules();
        self.overflow_ap = ap;

        if self.overflow_ap > rules.max_overflow_ap {
            self.overflow_ap = rules.max_overflow_ap;
        } else if self.overflow_ap < rules.min_overflow_ap {
            self.overflow_ap = rules.min_overflow_ap;
        }
    }

    pub fn change_overflow_ap(&mut self, ap: i32) {
        let rules = Module::rules();
        self.overflow_ap += ap;

        if self.overflow_ap > rules.max_overflow_ap {
            self.overflow_ap = rules.max_overflow_ap;
        } else if self.overflow_ap < rules.min_overflow_ap {
            self.overflow_ap = rules.min_overflow_ap;
        }
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

    pub(crate) fn add_hp(&mut self, hp: u32) {
        let hp = hp as i32;
        if hp + self.hp > self.stats.max_hp {
            self.hp = self.stats.max_hp;
        } else {
            self.hp += hp;
        }

        self.listeners.notify(&self);
    }

    pub fn check_removal(&mut self) {
        let start_len = self.effects.len();

        self.effects.retain(|e| !e.is_removal());

        if start_len != self.effects.len() {
            self.compute_stats();
        }
    }

    pub fn update(entity: &Rc<RefCell<EntityState>>, millis_elapsed: u32) -> Vec<Rc<ScriptCallback>> {
        let mut cbs_to_fire = Vec::new();

        {
            let actor = &mut entity.borrow_mut().actor;
            for effect in actor.effects.iter_mut() {
                if effect.update(millis_elapsed) {
                    // round timer changed for effect
                    cbs_to_fire.append(&mut effect.callbacks());
                }
            }

            for (_, ability_state) in actor.ability_states.iter_mut() {
                ability_state.update(millis_elapsed);
            }
        }

        entity.borrow_mut().actor.check_removal();

        cbs_to_fire
    }

    pub fn add_effect(&mut self, effect: Effect) {
        debug!("Adding effect with duration {} to '{}'", effect.duration_millis(),
            self.actor.name);

        self.effects.push(effect);
        self.compute_stats();
    }

    pub fn init(&mut self) {
        self.hp = self.stats.max_hp;
    }

    pub fn init_turn(&mut self) {
        let rules = Module::rules();

        let mut ap = rules.base_ap as i32 + self.overflow_ap;

        if ap < 0 {
            self.overflow_ap += rules.base_ap as i32;
        } else {
            self.overflow_ap = 0;
        }

        ap += self.stats.bonus_ap;
        if ap < 0 {
            ap = 0;
        }

        let mut ap = ap as u32;
        if ap > rules.max_ap {
            ap = rules.max_ap;
        }

        self.ap = ap;

        self.listeners.notify(&self);
    }

    pub fn init_actor_turn(entity: &Rc<RefCell<EntityState>>) {
        entity.borrow_mut().actor.init_turn();
    }

    pub fn end_turn(&mut self) {
        let max_overflow_ap = Module::rules().max_overflow_ap;
        self.overflow_ap += self.ap as i32;
        if self.overflow_ap > max_overflow_ap {
            self.overflow_ap = max_overflow_ap;
        }

        self.ap = 0;
        self.listeners.notify(&self);
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

        for ability in self.actor.abilities.iter() {
            self.stats.add(&ability.bonuses);
        }

        let mut attacks_list = Vec::new();
        for ref item_state in self.inventory.equipped_iter() {
            let equippable = match item_state.item.equippable {
                None => continue,
                Some(ref equippable) => {
                    if let Some(ref attack) = equippable.attack {
                        attacks_list.push(attack);
                    }

                    equippable
                }
            };

            self.stats.add(&equippable.bonuses);
        }

        let multiplier = if attacks_list.is_empty() {
            attacks_list.push(&self.actor.race.base_attack);
            1.0
        } else if attacks_list.len() > 1 {
            rules.dual_wield_damage_multiplier
        } else {
            1.0
        };

        for effect in self.effects.iter() {
            self.stats.add(effect.bonuses());
        }

        self.stats.finalize(attacks_list, multiplier, rules.base_attribute);
        self.stats.crit_threshold += rules.crit_percentile as i32;
        self.stats.hit_threshold += rules.hit_percentile as i32;
        self.stats.graze_threshold += rules.graze_percentile as i32;
        self.stats.graze_multiplier += rules.graze_damage_multiplier;
        self.stats.crit_multiplier += rules.crit_damage_multiplier;
        self.has_level_up = rules.get_xp_for_next_level(self.actor.total_level) <= self.xp;

        self.listeners.notify(&self);
    }
}
