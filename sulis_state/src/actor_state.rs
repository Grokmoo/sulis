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
use sulis_core::image::{LayeredImage};
use sulis_core::util::{invalid_data_error, ExtInt};
use sulis_rules::{AccuracyKind, Attack, AttackKind, BonusList, HitKind, StatList, WeaponKind,
    QuickSlot, Slot, ItemKind};
use sulis_module::{Actor, Module, ActorBuilder};
use area_feedback_text::ColorKind;
use {AbilityState, ChangeListenerList, Effect, EntityState, GameState, Inventory, ItemState, PStats};
use save_state::ActorSaveState;

pub struct ActorState {
    pub actor: Rc<Actor>,
    pub stats: StatList,
    pub listeners: ChangeListenerList<ActorState>,
    inventory: Inventory,
    effects: Vec<(usize, BonusList)>,
    image: LayeredImage,
    pub(crate) ability_states: HashMap<String, AbilityState>,
    texture_cache_invalid: bool,
    p_stats: PStats,
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
            if ability.ability.active.is_none() { continue; }

            let mut ability_state = AbilityState::new(&ability.ability);

            match save.ability_states.get(&ability.ability.id) {
                None => (),
                Some(ref ability_save) => {
                    ability_state.remaining_duration = ability_save.remaining_duration;
                }
            }

            ability_states.insert(ability.ability.id.to_string(), ability_state);
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
        };

        actor_state.compute_stats();

        for (slot, item) in actor.inventory.equipped_iter() {
            let item = ItemState::new(item);
            if !actor_state.inventory.can_equip(&item, &actor_state.stats, &actor) {
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

    pub fn current_uses_per_encounter(&self, ability_group: &str) -> ExtInt {
        *self.p_stats.current_group_uses_per_encounter
            .get(ability_group).unwrap_or(&ExtInt::Int(0))
    }

    pub fn ability_state(&mut self, id: &str) -> Option<&mut AbilityState> {
        self.ability_states.get_mut(id)
    }

    /// Returns true if the parent can swap weapons, false otherwise
    pub fn can_swap_weapons(&self) -> bool {
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

    /// Returns true if the ability state for the given ability can be
    /// activated (any active ability) or deactivated (only relevant for modes)
    pub fn can_toggle(&self, id: &str) -> bool {
        match self.ability_states.get(id) {
            None => false,
            Some(ref state) => {
                if self.p_stats.ap() < state.activate_ap() { return false; }

                if state.is_active_mode() { return true; }

                if self.current_uses_per_encounter(&state.group).is_zero() { return false; }

                state.is_available()
            }
        }
    }

    pub fn can_activate(&self, id: &str) -> bool {
        match self.ability_states.get(id) {
            None => false,
            Some(ref state) => {
                if self.p_stats.ap() < state.activate_ap() { return false; }

                if self.current_uses_per_encounter(&state.group).is_zero() { return false; }

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
        match self.ability_states.get_mut(id) {
            None => (),
            Some(ref mut state) => {
                state.activate();
                let cur = *self.p_stats.current_group_uses_per_encounter
                    .get(&state.group).unwrap_or(&ExtInt::Int(1));
                self.p_stats.current_group_uses_per_encounter
                    .insert(state.group.to_string(), cur - 1);
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

        self.compute_stats();
        self.init_day();
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

    pub(crate) fn has_ap_to_attack(&self) -> bool {
        self.p_stats.ap() >= self.stats.attack_cost as u32
    }

    pub fn weapon_attack(parent: &Rc<RefCell<EntityState>>,
                         target: &Rc<RefCell<EntityState>>) -> (HitKind, u32, String, ColorKind) {
        if target.borrow_mut().actor.hp() <= 0 {
            return (HitKind::Miss, 0, "Miss".to_string(), ColorKind::Miss);
        }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name, target.borrow().actor.actor.name);

        let mut color = ColorKind::Miss;
        let mut damage_str = String::new();
        let mut not_first = false;
        let mut hit_kind = HitKind::Miss;
        let mut total_damage = 0;

        let attacks = parent.borrow().actor.stats.attacks.clone();

        let mut is_flanking = false;
        let mgr = GameState::turn_manager();
        for entity in mgr.borrow().entity_iter() {
            if Rc::ptr_eq(&entity, parent) { continue; }
            if Rc::ptr_eq(&entity, target) { continue; }

            let entity = entity.borrow();
            if !entity.is_hostile(&target) { continue; }

            // TODO allow ranged weapons to flank?  and at any distance?
            if !entity.can_reach(&target) { continue; }

            let p_target = (target.borrow().center_x_f32(), target.borrow().center_y_f32());
            let p_parent = (parent.borrow().center_x_f32(), parent.borrow().center_y_f32());
            let p_other = (entity.center_x_f32(), entity.center_y_f32());

            let p1 = (p_target.0 - p_parent.0, p_target.1 - p_parent.1);
            let p2 = (p_target.0 - p_other.0, p_target.1 - p_other.1);

            let angle = ((p1.0 * p2.0 + p1.1 * p2.1) / (p1.0.hypot(p1.1) * p2.0.hypot(p2.1))).acos();
            let angle = angle.to_degrees();

            // info!("Got angle {} between {} and {} attacking {}", angle, parent.borrow().actor.actor.name,
            //     entity.actor.actor.name, target.borrow().actor.actor.name);
            if angle > parent.borrow().actor.stats.flanking_angle as f32 {
                is_flanking = true;
                break;
            }
        }

        for attack in attacks {
            if not_first { damage_str.push_str(", "); }

            let attack = if is_flanking {
                Attack::from(&attack, &parent.borrow().actor.stats.flanking_bonuses)
            } else {
                attack
            };

            let (hit, dmg, attack_result, attack_color) = ActorState::attack_internal(parent, target, &attack);
            if attack_color != ColorKind::Miss {
                color = attack_color;
            }

            damage_str.push_str(&attack_result);

            if hit > hit_kind {
                hit_kind = hit;
            }

            total_damage += dmg;

            not_first = true;
        }

        ActorState::check_death(parent, target);
        (hit_kind, total_damage, damage_str, color)
    }

    pub fn attack(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                  attack: &Attack) -> (HitKind, u32, String, ColorKind) {
        if target.borrow_mut().actor.hp() <= 0 {
            return (HitKind::Miss, 0, "Miss".to_string(), ColorKind::Miss);
        }

        info!("'{}' attacks '{}'", parent.borrow().actor.actor.name, target.borrow().actor.actor.name);

        let result = ActorState::attack_internal(parent, target, attack);

        ActorState::check_death(parent, target);
        result
    }

    fn attack_internal(parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                       attack: &Attack) -> (HitKind, u32, String, ColorKind) {
        let rules = Module::rules();

        let concealment = cmp::max(0, target.borrow().actor.stats.concealment -
                                   parent.borrow().actor.stats.concealment_ignore);

        if !rules.concealment_roll(concealment) {
            debug!("Concealment miss");
            return (HitKind::Miss, 0, "Concealment".to_string(), ColorKind::Miss);
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
                    return (HitKind::Hit, 0, "".to_string(), ColorKind::Miss);
                }
            }
        };

        let (hit_kind, damage_multiplier) = {
            let parent_stats = &parent.borrow().actor.stats;
            let hit_kind = parent_stats.attack_roll(accuracy_kind, defense, &attack.bonuses);
            let damage_multiplier = match hit_kind {
                HitKind::Miss => {
                    debug!("Miss");
                    return (HitKind::Miss, 0, "Miss".to_string(), ColorKind::Miss);
                },
                HitKind::Graze =>
                    parent_stats.graze_multiplier + attack.bonuses.graze_multiplier,
                HitKind::Hit =>
                    parent_stats.hit_multiplier + attack.bonuses.hit_multiplier,
                HitKind::Crit =>
                    parent_stats.crit_multiplier + attack.bonuses.crit_multiplier,
            };
            (hit_kind, damage_multiplier)
        };

        let damage = attack.roll_damage(&target.borrow().actor.stats.armor, damage_multiplier);

        debug!("{:?}. {:?} damage", hit_kind, damage);

        if !damage.is_empty() {
            let mut total = 0;
            for (_kind, amount) in damage {
                total += amount;
            }

            EntityState::remove_hp(target, parent, hit_kind, total);
            return (hit_kind, total, format!("{:?}: {}", hit_kind, total), ColorKind::Hit);
        } else if attack.damage.max() == 0 {
            // if attack cannot do any damage
            return (hit_kind, 0, format!("{:?}", hit_kind), ColorKind::Hit);
        } else {
            return (hit_kind, 0, format!("{:?}: {}", hit_kind, 0), ColorKind::Miss);
        }
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
        if !self.inventory.can_equip(&item, &self.stats, &self.actor) {
            return vec![item];
        }

        let unequipped = self.inventory.equip(item, preferred_slot);
        self.compute_stats();
        self.texture_cache_invalid = true;
        unequipped
    }

    #[must_use]
    pub fn unequip(&mut self, slot: Slot) -> Option<ItemState> {
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
        self.p_stats.has_level_up()
    }

    pub fn add_xp(&mut self, xp: u32) {
        self.p_stats.add_xp(xp, &self.actor);
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

        let layers = self.actor.image_layers().get_list_with(self.actor.sex, &self.actor.race,
                                                             self.actor.hair_color, self.actor.skin_color,
                                                             self.inventory.get_image_layers());
        self.image = LayeredImage::new(layers, self.actor.hue);

        let rules = Module::rules();
        self.stats.add(&self.actor.race.base_stats);

        for &(ref class, level) in self.actor.levels.iter() {
            self.stats.add_multiple(&class.bonuses_per_level, level);
            for (ref group_id, amount) in class.group_uses_per_encounter(level).iter() {
                self.stats.add_single_group_uses_per_encounter(group_id, *amount);
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

        self.stats.finalize(attacks_list, equipped_armor, weapon_style, multiplier, rules.base_attribute);
        self.stats.flanking_angle += rules.base_flanking_angle;
        self.stats.crit_threshold += rules.crit_percentile as i32;
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
