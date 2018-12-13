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

use std::cmp;
use std::io::Error;
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::HashMap;

use sulis_core::io::GraphicsRenderer;
use sulis_core::image::{LayeredImage, Image};
use sulis_core::util::{invalid_data_error, ExtInt};
use sulis_rules::{AccuracyKind, Attack, AttackKind, BonusList, HitKind, StatList, WeaponKind,
    QuickSlot, Slot, ItemKind, DamageKind, HitFlags, WeaponStyle};
use sulis_module::{Actor, Module, ActorBuilder, Faction, ImageLayer};
use crate::{AbilityState, ChangeListenerList, Effect, EntityState, GameState, Inventory, ItemState, PStats};
use crate::save_state::ActorSaveState;

pub struct ActorState {
    pub actor: Rc<Actor>,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
    inventory: Inventory,
    effects: Vec<(usize, BonusList)>,
    image: LayeredImage,
    pub(crate) ability_states: HashMap<String, AbilityState>,
    texture_cache_invalid: bool,
    anim_image_layers: HashMap<ImageLayer, Rc<Image>>,
    p_stats: PStats,
}

impl ActorState {
    pub fn load(mut save: ActorSaveState, base: Option<ActorBuilder>) -> Result<ActorState, Error> {
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
            if ability.ability.active.is_none() { continue; }

            let mut ability_state = AbilityState::new(&ability.ability);

            match save.ability_states.remove(&ability.ability.id) {
                None => (),
                Some(ability_save) => {
                    ability_state.remaining_duration = ability_save.remaining_duration;
                }
            }

            ability_states.insert(ability.ability.id.to_string(), ability_state);
        }

        // Add any abilities that aren't on the base actor
        for (ability_id, state) in save.ability_states {
            let ability = match Module::ability(&ability_id) {
                None => return invalid_data_error(&format!("No ability with ID '{}' for actor '{}'",
                                                    ability_id, actor.id)),
                Some(ability) => ability,
            };

            let mut ability_state = AbilityState::new(&ability);
            ability_state.remaining_duration = state.remaining_duration;
            ability_states.insert(ability_id, ability_state);
        }

        let mut inventory = Inventory::empty();
        inventory.load(save.equipped, save.quick)?;

        Ok(ActorState {
            actor,
            inventory,
            stats: StatList::new(attrs),
            listeners: ChangeListenerList::default(),
            image,
            effects: Vec::new(),
            ability_states,
            texture_cache_invalid: false,
            p_stats: save.p_stats,
            anim_image_layers: HashMap::new(),
        })
    }

    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let inventory = Inventory::empty();

        let image = LayeredImage::new(actor.image_layers().get_list(actor.sex,
                                                                    actor.hair_color,
                                                                    actor.skin_color), actor.hue);
        let attrs = actor.attributes;

        let mut ability_states = HashMap::new();
        for ability in actor.abilities.iter() {
            let ability = &ability.ability;
            if ability.active.is_none() { continue; }

            ability_states.insert(ability.id.to_string(), AbilityState::new(ability));
        }

        let mut actor_state = ActorState {
            actor: Rc::clone(&actor),
            inventory,
            stats: StatList::new(attrs),
            listeners: ChangeListenerList::default(),
            image,
            effects: Vec::new(),
            ability_states,
            texture_cache_invalid: false,
            p_stats: PStats::new(&actor),
            anim_image_layers: HashMap::new(),
        };

        actor_state.compute_stats();

        for (slot, item) in actor.inventory.equipped_iter() {
            let item = ItemState::new(item);
            if !actor_state.can_equip(&item) {
                warn!("Unable to equip item '{}' for actor '{}'", item.item.id, actor.id);
            } else {
                let _ = actor_state.inventory.equip(item, Some(slot));
                // don't deal with any items which have been unequiped as a result
            }
        }

        for (slot, item) in actor.inventory.quick_iter() {
            let item = ItemState::new(item);
            if !actor_state.inventory.can_set_quick(&item, slot, &actor) {
                warn!("Unable to set quick item '{}' for actor '{}'", item.item.id, actor.id);
            } else {
                let _ = actor_state.inventory.set_quick(item, slot);
                // don't deal with any item which has been removed as a result
            }
        }

        actor_state
    }

    pub fn add_anim_image_layers(&mut self, images: &HashMap<ImageLayer, Rc<Image>>) {
        let mut change = false;
        for (layer, ref image) in images.iter() {
            if let Some(img) = self.anim_image_layers.get(layer) {
                if Rc::ptr_eq(img, image) { continue; }
            }
            change = true;
            self.anim_image_layers.insert(*layer, Rc::clone(image));
        }

        if change {
            self.texture_cache_invalid = true;
            self.compute_stats();
        }
    }

    pub fn remove_anim_image_layers(&mut self, images: &HashMap<ImageLayer, Rc<Image>>) {
        for ref layer in images.keys() {
            self.anim_image_layers.remove(*layer);
        }

        self.texture_cache_invalid = true;
        self.compute_stats();
    }

    pub fn is_inventory_locked(&self) -> bool { self.p_stats.is_inventory_locked() }

    pub fn set_inventory_locked(&mut self, locked: bool) {
        self.p_stats.set_inventory_locked(locked);
    }

    pub fn is_threatened(&self) -> bool { self.p_stats.is_threatened() }

    pub fn add_threatening(&mut self, index: usize) {
        self.p_stats.add_threatening(index);
    }

    pub fn remove_threatening(&mut self, index: usize) {
        self.p_stats.remove_threatening(index);
    }

    pub fn add_threatener(&mut self, index: usize) {
        let cur = self.p_stats.is_threatened();
        self.p_stats.add_threatener(index);

        if cur != self.p_stats.is_threatened() {
            trace!("Recompute threat on {}", self.actor.name);
            self.compute_stats();
        }
    }

    pub fn remove_threatener(&mut self, index: usize) {
        let cur = self.p_stats.is_threatened();
        self.p_stats.remove_threatener(index);

        if cur != self.p_stats.is_threatened() {
            trace!("Recompute threat on {}", self.actor.name);
            self.compute_stats();
        }
    }

    pub fn faction(&self) -> Faction {
        self.p_stats.faction
    }

    pub fn set_faction(&mut self, faction: Faction) {
        self.p_stats.faction = faction;
    }

    pub fn clone_p_stats(&self) -> PStats {
        self.p_stats.clone()
    }

    pub fn check_texture_cache_invalid(&mut self) -> bool {
        if self.texture_cache_invalid {
            self.texture_cache_invalid = false;
            true
        } else {
            false
        }
    }

    pub fn current_uses_per_day(&self, ability_group: &str) -> ExtInt {
        *self.p_stats.current_group_uses_per_day
            .get(ability_group).unwrap_or(&ExtInt::Int(0))
    }

    pub fn current_uses_per_encounter(&self, ability_group: &str) -> ExtInt {
        *self.p_stats.current_group_uses_per_encounter
            .get(ability_group).unwrap_or(&ExtInt::Int(0))
    }

    pub fn ability_state(&mut self, id: &str) -> Option<&mut AbilityState> {
        self.ability_states.get_mut(id)
    }

    /// Returns true if the parent can swap weapons, false otherwise
    pub fn can_swap_weapons(&self) -> bool {
        if self.p_stats.is_inventory_locked() { return false; }

        self.p_stats.ap() >= Module::rules().swap_weapons_ap
    }

    /// Returns true if this actor can use the item in the specified quick slot
    /// now - which includes having sufficient AP, false otherwise
    pub fn can_use_quick(&self, slot: QuickSlot) -> bool {
        match self.inventory.quick(slot) {
            None => false,
            Some(ref item) => self.can_use(item),
        }
    }

    /// Returns true if this actor can use the item at some point - not
    /// taking AP into consideration, false otherwise
    pub fn can_use_sometime(&self, item_state: &ItemState) -> bool {
        if !item_state.item.usable.is_some() { return false; }

        if !item_state.item.meets_prereqs(&self.actor) { return false; }

        true
    }

    /// Returns true if the specified item can be used now - which includes
    /// having sufficient AP, false otherwise
    pub fn can_use(&self, item_state: &ItemState) -> bool {
        if !item_state.item.meets_prereqs(&self.actor) { return false; }

        match &item_state.item.usable {
            None => false,
            Some(usable) => {
                self.p_stats.ap() >= usable.ap
            }
        }
    }

    fn group_has_uses(&self, group_id: &str) -> bool {
        if self.current_uses_per_encounter(group_id).greater_than(0) { return true; }

        self.current_uses_per_day(group_id).greater_than(0)
    }

    /// Returns true if the ability state for the given ability can be
    /// activated (any active ability) or deactivated (only relevant for modes)
    pub fn can_toggle(&self, id: &str) -> bool {
        if self.stats.abilities_disabled { return false; }

        match self.ability_states.get(id) {
            None => false,
            Some(ref state) => {
                if self.p_stats.ap() < state.activate_ap() { return false; }

                if state.is_active_mode() { return true; }

                if !self.group_has_uses(&state.group) { return false; }

                state.is_available()
            }
        }
    }

    pub fn can_activate(&self, id: &str) -> bool {
        if self.stats.abilities_disabled { return false; }

        match self.ability_states.get(id) {
            None => false,
            Some(ref state) => {
                if self.p_stats.ap() < state.activate_ap() { return false; }

                if !self.group_has_uses(&state.group) { return false; }

                state.is_available()
            }
        }
    }

    pub fn deactivate_ability_state(&mut self, id: &str) {
        match self.ability_states.get_mut(id) {
            None => (),
            Some(ref mut state) => {
                state.deactivate();

                let mgr = GameState::turn_manager();
                let mut mgr = mgr.borrow_mut();

                for (index, _) in self.effects.iter() {
                    let effect = mgr.effect_mut(*index);
                    if effect.deactivates_with(id) {
                        effect.mark_for_removal();
                    }
                }
            }
        }
    }

    pub fn activate_ability_state(&mut self, id: &str) {
        let state = match self.ability_states.get_mut(id) {
            None => return,
            Some(state) => state
        };
        state.activate();

        let decrement_uses = !self.stats.free_ability_group_use;

        if decrement_uses {
            let per_enc = *self.p_stats.current_group_uses_per_encounter
                .get(&state.group).unwrap_or(&ExtInt::Int(0));

            // take one use from per encounter if available, otherwise take from per day
            if per_enc.is_zero() {
                let per_day = *self.p_stats.current_group_uses_per_day
                    .get(&state.group).unwrap_or(&ExtInt::Int(0));
                self.p_stats.current_group_uses_per_day
                    .insert(state.group.to_string(), per_day - 1);
            } else {
                self.p_stats.current_group_uses_per_encounter
                    .insert(state.group.to_string(), per_enc - 1);
            }
        }
    }

    pub fn effects_iter<'a>(&'a self) -> impl Iterator<Item=&'a usize> {
        self.effects.iter().map(|(index, _)| index)
    }

    pub fn replace_actor(&mut self, new_actor: Actor) {
        self.actor = Rc::new(new_actor);

        for ability in self.actor.abilities.iter() {
            let ability = &ability.ability;
            if ability.active.is_none() { continue; }
            if self.ability_states.contains_key(&ability.id) { continue; }

            self.ability_states.insert(ability.id.to_string(), AbilityState::new(ability));
        }

        let mut to_remove = Vec::new();
        for id in self.ability_states.keys() {
            let mut found = false;
            for owned in self.actor.abilities.iter() {
                if &owned.ability.id == id {
                    found = true;
                    break;
                }
            }

            if !found { to_remove.push(id.clone()); }
        }

        for id in to_remove {
            self.ability_states.remove(&id);
        }

        self.compute_stats();
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

        if !self.has_ap_to_attack() { return false; }

        self.can_reach(dist)
    }

    pub fn has_ap_to_attack(&self) -> bool {
        self.p_stats.ap() >= self.stats.attack_cost as u32
    }

    fn is_sneak_attack(parent: &Rc<RefCell<EntityState>>,
                       target: &Rc<RefCell<EntityState>>) -> bool {
        parent.borrow().actor.stats.hidden && !target.borrow().actor.stats.sneak_attack_immunity
    }

    fn is_flanking(parent: &Rc<RefCell<EntityState>>,
                   target: &Rc<RefCell<EntityState>>) -> bool {
        if target.borrow().actor.stats.flanked_immunity { return false; }

        let mgr = GameState::turn_manager();
        let area = GameState::get_area_state(&parent.borrow().location.area_id).unwrap();
        let area = area.borrow();
        for entity_index in area.entity_iter() {
            let entity = mgr.borrow().entity(*entity_index);

            if Rc::ptr_eq(&entity, parent) { continue; }
            if Rc::ptr_eq(&entity, target) { continue; }

            let entity = entity.borrow();

            if !entity.is_hostile(&target) { continue; }

            if entity.actor.stats.attack_disabled { continue; }

            match entity.actor.inventory.weapon_style() {
                WeaponStyle::Ranged => continue,
                WeaponStyle::TwoHanded | WeaponStyle::Single
                    | WeaponStyle::Shielded | WeaponStyle::DualWielding => (),
            }

            if !entity.can_reach(&target) { continue; }

            let p_target = (target.borrow().center_x_f32(), target.borrow().center_y_f32());
            let p_parent = (parent.borrow().center_x_f32(), parent.borrow().center_y_f32());
            let p_other = (entity.center_x_f32(), entity.center_y_f32());

            let p1 = (p_target.0 - p_parent.0, p_target.1 - p_parent.1);
            let p2 = (p_target.0 - p_other.0, p_target.1 - p_other.1);

            let mut cos_angle = (p1.0 * p2.0 + p1.1 * p2.1) / (p1.0.hypot(p1.1) * p2.0.hypot(p2.1));
            if cos_angle > 1.0 { cos_angle = 1.0; }
            if cos_angle < -1.0 { cos_angle = -1.0; }

            let angle = cos_angle.acos().to_degrees();

            debug!("Got angle {} between {} and {} attacking {}", angle,
                   parent.borrow().actor.actor.name, entity.actor.actor.name,
                   target.borrow().actor.actor.name);

            if angle > parent.borrow().actor.stats.flanking_angle as f32 {
                return true;
            }
        }

        false
    }

    pub fn weapon_attack(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>)
        -> Vec<(HitKind, HitFlags, Vec<(DamageKind, u32)>)> {

        if target.borrow_mut().actor.hp() <= 0 {
            return vec![(HitKind::Miss, HitFlags::default(), Vec::new())];
        }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name, target.borrow().actor.actor.name);

        let attacks = parent.borrow().actor.stats.attacks.clone();

        let is_flanking = ActorState::is_flanking(parent, target);
        let is_sneak_attack = ActorState::is_sneak_attack(parent, target);

        let mut result = Vec::new();
        for attack in attacks {
            let mut attack = if is_flanking {
                Attack::from(&attack, &parent.borrow().actor.stats.flanking_bonuses)
            } else {
                attack
            };

            let (hit_kind, hit_flags, damage) =
                ActorState::attack_internal(parent, target, &mut attack,
                                            is_flanking, is_sneak_attack);
            result.push((hit_kind, hit_flags, damage));
        }

        ActorState::check_death(parent, target);
        result
    }

    pub fn attack(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                  attack: &mut Attack) -> (HitKind, HitFlags, Vec<(DamageKind, u32)>) {
        if target.borrow_mut().actor.hp() <= 0 {
            return (HitKind::Miss, HitFlags::default(), Vec::new());
        }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name,
            target.borrow().actor.actor.name);

        let is_flanking = ActorState::is_flanking(parent, target);
        let is_sneak_attack = ActorState::is_sneak_attack(parent, target);

        let (hit_kind, hit_flags, damage) =
            ActorState::attack_internal(parent, target, attack, is_flanking, is_sneak_attack);

        ActorState::check_death(parent, target);

        (hit_kind, hit_flags, damage)
    }

    fn attack_internal(parent: &Rc<RefCell<EntityState>>,
                       target: &Rc<RefCell<EntityState>>,
                       attack: &mut Attack,
                       flanking: bool,
                       sneak_attack: bool) -> (HitKind, HitFlags, Vec<(DamageKind, u32)>) {
        let rules = Module::rules();

        let concealment = cmp::max(0, target.borrow().actor.stats.concealment -
                                   parent.borrow().actor.stats.concealment_ignore);

        if !rules.concealment_roll(concealment) {
            debug!("Concealment miss");
            return (HitKind::Miss, HitFlags { concealment: true, ..Default::default() },
                Vec::new());
        }

        let (accuracy_kind, defense) = {
            let target_stats = &target.borrow().actor.stats;
            match attack.kind {
                AttackKind::Fortitude { accuracy } => (accuracy, target_stats.fortitude),
                AttackKind::Reflex { accuracy } => (accuracy, target_stats.reflex),
                AttackKind::Will { accuracy } => (accuracy, target_stats.will),
                AttackKind::Melee { .. } => (AccuracyKind::Melee, target_stats.defense),
                AttackKind::Ranged { .. } => (AccuracyKind::Ranged, target_stats.defense),
                AttackKind::Dummy => {
                    return (HitKind::Hit, HitFlags::default(), Vec::new());
                }
            }
        };
        let crit_immunity = target.borrow().actor.stats.crit_immunity;

        if flanking {
            attack.bonuses.melee_accuracy += rules.flanking_accuracy_bonus;
            attack.bonuses.ranged_accuracy += rules.flanking_accuracy_bonus;
            attack.bonuses.spell_accuracy += rules.flanking_accuracy_bonus;
        } else if sneak_attack {
            attack.bonuses.melee_accuracy += rules.hidden_accuracy_bonus;
            attack.bonuses.ranged_accuracy += rules.hidden_accuracy_bonus;
            attack.bonuses.spell_accuracy += rules.hidden_accuracy_bonus;
        }

        let hit_flags = HitFlags { flanking, sneak_attack, concealment: false };

        let (hit_kind, damage_multiplier) = {
            let parent_stats = &parent.borrow().actor.stats;
            let hit_kind = parent_stats.attack_roll(accuracy_kind, crit_immunity,
                                                    defense, &attack.bonuses);
            let damage_multiplier = match hit_kind {
                HitKind::Miss => {
                    debug!("Miss");
                    return (HitKind::Miss, hit_flags, Vec::new());
                },
                HitKind::Graze =>
                    parent_stats.graze_multiplier + attack.bonuses.graze_multiplier,
                HitKind::Hit =>
                    parent_stats.hit_multiplier + attack.bonuses.hit_multiplier,
                HitKind::Crit =>
                    parent_stats.crit_multiplier + attack.bonuses.crit_multiplier,
                HitKind::Auto => panic!(),
            };
            (hit_kind, damage_multiplier)
        };

        let damage = {
            let target = &target.borrow().actor.stats;
            let damage = &attack.damage;
            rules.roll_damage(damage, &target.armor, &target.resistance, damage_multiplier)
        };

        debug!("{:?}. {:?} damage", hit_kind, damage);

        if !damage.is_empty() {
            let mut total = 0;
            for (_kind, amount) in damage.iter() {
                total += amount;
            }

            EntityState::remove_hp(target, parent, hit_kind, damage.clone());
        }

        return (hit_kind, hit_flags, damage);
    }

    /// Sets the specified item as the item at the quick slot.  Returns the
    /// item that was previously there, if it was present
    #[must_use]
    pub fn set_quick(&mut self, item: ItemState, slot: QuickSlot) -> Option<ItemState> {
        let item = self.inventory.set_quick(item, slot);
        self.listeners.notify(&self);
        item
    }

    /// Clears any item at the specified quick slot.  Returns the item
    /// if it is present
    #[must_use]
    pub fn clear_quick(&mut self, slot: QuickSlot) -> Option<ItemState> {
        let item = self.inventory.clear_quick(slot);
        self.listeners.notify(&self);
        item
    }

    pub fn swap_weapon_set(&mut self) {
        let swap_ap = Module::rules().swap_weapons_ap;
        if self.ap() < swap_ap { return; }
        self.inventory.swap_weapon_set();
        self.compute_stats();
        self.texture_cache_invalid = true;
        if GameState::is_combat_active() {
            self.remove_ap(swap_ap);
        }
    }

    /// Attempts to equip the specified item to this actor's inventory.
    /// Returns a list of free items that need to be placed somewhere.
    /// If the equip action was not possible, this will include the item that was
    /// passed in.  Otherwise, it will include any items that were unequipped
    /// in order to equip the new item.  This will frequently be an empty list
    #[must_use]
    pub fn equip(&mut self, item: ItemState, preferred_slot: Option<Slot>) -> Vec<ItemState> {
        if !self.can_equip(&item) {
            return vec![item];
        }

        let unequipped = self.inventory.equip(item, preferred_slot);
        self.compute_stats();
        self.texture_cache_invalid = true;
        unequipped
    }

    pub fn can_equip(&self, item: &ItemState) -> bool {
        if self.p_stats.is_inventory_locked() { return false; }

        self.inventory.can_equip(&item, &self.stats, &self.actor)
    }

    pub fn can_unequip(&self, _slot: Slot) -> bool {
        if self.p_stats.is_inventory_locked() { return false; }

        return !GameState::is_combat_active()
    }

    #[must_use]
    pub fn unequip(&mut self, slot: Slot) -> Option<ItemState> {
        if self.p_stats.is_inventory_locked() { return None; }

        let item = self.inventory.unequip(slot);
        self.compute_stats();
        self.texture_cache_invalid = true;
        item
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn is_dead(&self) -> bool {
        self.hp() <= 0
    }

    pub fn check_death(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {
        if target.borrow().actor.hp() > 0 { return; }

        let area_state = GameState::area_state();

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

    pub fn is_disabled(&self) -> bool { self.p_stats.is_disabled() }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.p_stats.set_disabled(disabled);
    }

    pub fn has_level_up(&self) -> bool {
        self.p_stats.has_level_up()
    }

    pub fn add_xp(&mut self, xp: u32) {
        self.p_stats.add_xp(xp, &self.actor);
        self.listeners.notify(&self);
    }

    pub fn xp(&self) -> u32 {
        self.p_stats.xp()
    }

    pub fn hp(&self) -> i32 {
        self.p_stats.hp()
    }

    pub fn overflow_ap(&self) -> i32 {
        self.p_stats.overflow_ap()
    }

    pub fn ap(&self) -> u32 {
        self.p_stats.ap()
    }

    pub fn get_move_ap_cost(&self, squares: u32) -> u32 {
        let rules = Module::rules();
        ((rules.movement_ap as f32) / self.stats.movement_rate) as u32 * squares
    }

    pub fn set_overflow_ap(&mut self, ap: i32) {
        self.p_stats.set_overflow_ap(ap);
    }

    pub fn change_overflow_ap(&mut self, ap: i32) {
        let cur_overflow = self.p_stats.overflow_ap();
        self.p_stats.set_overflow_ap(cur_overflow + ap);
    }

    pub(crate) fn add_ap(&mut self, ap: u32) {
        self.p_stats.add_ap(ap);
        self.listeners.notify(&self);
    }

    pub(crate) fn remove_ap(&mut self, ap: u32) {
        self.p_stats.remove_ap(ap);
        self.listeners.notify(&self);
    }

    pub(crate) fn remove_hp(&mut self, hp: u32) {
        self.p_stats.remove_hp(hp);
        self.listeners.notify(&self);
    }

    pub(crate) fn add_hp(&mut self, hp: u32) {
        self.p_stats.add_hp(hp, self.stats.max_hp);
        self.listeners.notify(&self);
    }

    pub fn elapse_time(&mut self, millis_elapsed: u32, all_effects: &Vec<Option<Effect>>) {
        for (_, ability_state) in self.ability_states.iter_mut() {
            ability_state.update(millis_elapsed);
        }

        let start_len = self.effects.len();
        self.effects.retain(|(index, _)| {
            all_effects[*index].is_some()
        });

        if start_len != self.effects.len() {
            self.compute_stats();
        }
    }

    pub fn add_effect(&mut self, index: usize, bonuses: BonusList) {
        info!("Adding effect with index {} to '{}'", index, self.actor.name);
        self.effects.push((index, bonuses));
        self.compute_stats();
    }

    pub (crate) fn remove_effect(&mut self, index: usize) {
        self.effects.retain(|(i, _)| *i != index);
        self.compute_stats();
    }

    pub fn init_day(&mut self) {
        self.p_stats.init_day(&self.stats);
        self.listeners.notify(&self);
    }

    pub fn end_encounter(&mut self) {
        self.p_stats.end_encounter(&self.stats);
        self.listeners.notify(&self);
    }

    pub fn init_turn(&mut self) {
        info!("Init turn for '{}' with overflow ap of {}", self.actor.name, self.overflow_ap());
        self.p_stats.init_turn(&self.stats);
        self.listeners.notify(&self);
    }

    pub fn end_turn(&mut self) {
        self.p_stats.end_turn();
        self.listeners.notify(&self);
    }

    pub fn compute_stats(&mut self) {
        debug!("Compute stats for '{}'", self.actor.name);
        self.stats = StatList::new(self.actor.attributes);

        let mut layers_override = self.inventory().get_image_layers();
        for (layer, image) in self.anim_image_layers.iter() {
            layers_override.insert(*layer, Rc::clone(image));
        }

        let layers = self.actor.image_layers().get_list_with(self.actor.sex,
                                                             &self.actor.race,
                                                             self.actor.hair_color,
                                                             self.actor.skin_color,
                                                             layers_override);
        self.image = LayeredImage::new(layers, self.actor.hue);

        let rules = Module::rules();
        self.stats.add(&self.actor.race.base_stats);

        for &(ref class, level) in self.actor.levels.iter() {
            self.stats.add_multiple(&class.bonuses_per_level, level);
            for (ref group_id, amount) in class.group_uses_per_encounter(level).iter() {
                self.stats.add_single_group_uses_per_encounter(group_id, *amount);
            }

            for (ref group_id, amount) in class.group_uses_per_day(level).iter() {
                self.stats.add_single_group_uses_per_day(group_id, *amount);
            }
        }

        for ability in self.actor.abilities.iter() {
            let level = ability.level;
            ability.ability.add_bonuses_to(level, &mut self.stats);
        }

        let mut attacks_list = Vec::new();
        for ref item_state in self.inventory.equipped_iter() {
            let equippable = match item_state.item.equippable {
                None => continue,
                Some(ref equippable) => {
                    if let Some(ref attack) = equippable.attack {
                        let weapon_kind = match item_state.item.kind {
                            ItemKind::Weapon { kind } => kind,
                            _ => {
                                warn!("Weapon attack belonging to item '{}' with no associated WeaponKind",
                                      item_state.item.id);
                                continue;
                            }
                        };

                        attacks_list.push((attack, weapon_kind));
                    }

                    equippable
                }
            };

            self.stats.add(&equippable.bonuses);
        }

        let multiplier = if attacks_list.is_empty() {
            attacks_list.push((&self.actor.race.base_attack, WeaponKind::Simple));
            1.0
        } else if attacks_list.len() > 1 {
            rules.dual_wield_damage_multiplier
        } else {
            1.0
        };

        for (_, ref bonuses) in self.effects.iter() {
            self.stats.add(bonuses);
        }

        let mut equipped_armor = HashMap::new();
        for slot in Slot::iter() {
            if let Some(ref item_state) = self.inventory.equipped(*slot) {
                match item_state.item.kind {
                    ItemKind::Armor { kind } => { equipped_armor.insert(*slot, kind); }
                    _ => (),
                }
            }
        }

        let weapon_style = self.inventory.weapon_style();
        let is_threatened = self.is_threatened();

        self.stats.finalize(attacks_list, equipped_armor, weapon_style,
                            multiplier, rules.base_attribute, is_threatened);
        self.stats.flanking_angle += rules.base_flanking_angle;
        self.stats.crit_chance += rules.crit_chance as i32;
        self.stats.hit_threshold += rules.hit_percentile as i32;
        self.stats.graze_threshold += rules.graze_percentile as i32;
        self.stats.graze_multiplier += rules.graze_damage_multiplier;
        self.stats.hit_multiplier += 1.0;
        self.stats.crit_multiplier += rules.crit_damage_multiplier;
        self.stats.movement_rate += self.actor.race.movement_rate;
        self.stats.attack_cost += rules.attack_ap as i32;

        self.p_stats.recompute_level_up(&self.actor);

        self.listeners.notify(&self);
    }
}
