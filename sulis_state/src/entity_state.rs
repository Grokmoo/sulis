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

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::ptr;
use std::rc::Rc;
use std::usize;

use sulis_core::config::Config;

use crate::animation::{self, Anim};
use crate::save_state::EntitySaveState;
use crate::script::{self, CallbackData, ScriptEntitySet};
use crate::{
    entity_attack_handler::weapon_attack, entity_texture_cache::Slot, is_within_attack_dist,
    ActorState, AreaState, ChangeListenerList, EntityTextureCache, EntityTextureSlot, GameState,
    Location, ScriptCallback, TurnManager,
};
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{color, Color};
use sulis_core::util::{invalid_data_error, Offset, Scale, Size, Point};
use sulis_module::area::MAX_AREA_SIZE;
use sulis_module::{
    actor::Faction, ai, Actor, DamageKind, HitKind, Module, ObjectSize, ObjectSizeIterator,
};

enum AIState {
    Player { vis: Vec<bool>, show_portrait: bool },
    AI { group: Option<usize>, active: bool },
}

pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<ObjectSize>,
    pub sub_pos: (f32, f32),
    pub color: Color,
    pub color_sec: Color,
    pub scale: f32,
    pub listeners: ChangeListenerList<EntityState>,

    ai_state: AIState,
    ai_callbacks: Option<Rc<CallbackData>>,
    pub(crate) marked_for_removal: bool,
    texture_cache_slot: Option<EntityTextureSlot>,

    custom_flags: HashMap<String, String>,

    index: usize,      // index in vec of the owning manager
    unique_id: String, // assigned when setting the index and persisted on save

    collapsed_groups: Vec<String>,
}

impl PartialEq for EntityState {
    fn eq(&self, other: &EntityState) -> bool {
        self.location.area_id == other.location.area_id && self.index == other.index
    }
}

impl EntityState {
    pub(crate) fn load(
        save: EntitySaveState,
        areas: &HashMap<String, Rc<RefCell<AreaState>>>,
    ) -> Result<EntityState, Error> {
        let ai_state = match save.actor_base.as_ref() {
            None => AIState::AI {
                group: save.ai_group,
                active: save.ai_active,
            },
            Some(_) => {
                let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
                AIState::Player {
                    vis: vec![false; dim],
                    show_portrait: save.show_portrait,
                }
            }
        };

        let area = match areas.get(&save.location.area) {
            None => {
                invalid_data_error(&format!("Invalid area '{}' for entity", save.location.area))
            }
            Some(area) => Ok(area),
        }?;

        let location = Location::new(save.location.x, save.location.y, &area.borrow().area.area);

        let size = match Module::object_size(&save.size) {
            None => invalid_data_error(&format!("Invalid size '{}' for actor", save.size)),
            Some(size) => Ok(size),
        }?;

        let actor = ActorState::load(save.actor, save.actor_base)?;

        Ok(EntityState {
            actor,
            ai_callbacks: None,
            location,
            size,
            index: save.index,
            unique_id: save.unique_id,
            sub_pos: (0.0, 0.0),
            color: color::WHITE,
            color_sec: Color::new(0.0, 0.0, 0.0, 0.0),
            scale: 1.0,
            listeners: ChangeListenerList::default(),
            ai_state,
            marked_for_removal: false,
            texture_cache_slot: None,
            custom_flags: save.custom_flags,
            collapsed_groups: save.collapsed_groups,
        })
    }

    pub(crate) fn new(
        actor: Rc<Actor>,
        unique_id: Option<String>,
        location: Location,
        is_pc: bool,
        ai_group: Option<usize>,
    ) -> EntityState {
        let ai_state = if is_pc {
            let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
            AIState::Player {
                vis: vec![false; dim],
                show_portrait: true,
            }
        } else {
            AIState::AI {
                group: ai_group,
                active: false,
            }
        };

        let unique_id = match unique_id {
            None => "".to_string(),
            Some(val) => val,
        };

        debug!("Creating new entity state for {}", actor.id);
        let size = Rc::clone(&actor.race.size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state,
            ai_callbacks: None,
            location,
            sub_pos: (0.0, 0.0),
            color: color::WHITE,
            color_sec: Color::new(0.0, 0.0, 0.0, 0.0),
            scale: 1.0,
            size,
            index: usize::MAX,
            unique_id,
            listeners: ChangeListenerList::default(),
            marked_for_removal: false,
            ai_state,
            texture_cache_slot: None,
            custom_flags: HashMap::new(),
            collapsed_groups: Vec::new(),
        }
    }

    pub fn add_collapsed_group(&mut self, group: String) {
        self.collapsed_groups.push(group);
    }

    pub fn remove_collapsed_group(&mut self, group: &str) {
        self.collapsed_groups.retain(|g| g != group);
    }

    pub fn collapsed_groups(&self) -> Vec<String> {
        self.collapsed_groups.clone()
    }

    pub fn unique_id(&self) -> &str {
        &self.unique_id
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
        if let Some(ref ai) = self.actor.actor.ai {
            let mut cbs = CallbackData::new_entity(self.index);
            for (kind, func) in ai.hooks.iter() {
                let func = func.to_string();
                match kind {
                    ai::FuncKind::OnDamaged => cbs.add_func(script::FuncKind::OnDamaged, func),
                    ai::FuncKind::BeforeAttack => {
                        cbs.add_func(script::FuncKind::BeforeAttack, func)
                    }
                    ai::FuncKind::AfterAttack => cbs.add_func(script::FuncKind::AfterAttack, func),
                    ai::FuncKind::BeforeDefense => {
                        cbs.add_func(script::FuncKind::BeforeDefense, func)
                    }
                    ai::FuncKind::OnRoundElapsed => {
                        cbs.add_func(script::FuncKind::OnRoundElapsed, func)
                    },
                    ai::FuncKind::AiAction => (), // this is handled specially when running the AI
                }
            }
            self.ai_callbacks = Some(Rc::new(cbs));
        }

        if self.unique_id.is_empty() {
            self.unique_id = format!("__uid__{}{}", self.actor.actor.id, index);
        }
    }

    pub fn ai_callbacks(&self) -> Option<Rc<CallbackData>> {
        self.ai_callbacks.clone()
    }

    pub fn callbacks(&self, mgr: &TurnManager) -> Vec<Rc<CallbackData>> {
        let mut result: Vec<_> = self
            .actor
            .effects_iter()
            .flat_map(|index| {
                if let Some(effect) = mgr.effect_checked(*index) {
                    effect.callbacks()
                } else {
                    Vec::new()
                }
            })
            .collect();

        if let Some(ref cb) = self.ai_callbacks {
            result.push(Rc::clone(cb));
        }

        result
    }

    pub fn custom_flags(&self) -> impl Iterator<Item = (&String, &String)> {
        self.custom_flags.iter()
    }

    pub fn clear_custom_flag(&mut self, flag: &str) {
        self.custom_flags.remove(flag);
    }

    pub fn set_custom_flag(&mut self, flag: &str, value: &str) {
        self.custom_flags
            .insert(flag.to_string(), value.to_string());
    }

    pub fn get_custom_flag(&self, flag: &str) -> Option<String> {
        self.custom_flags.get(flag).cloned()
    }

    pub fn add_num_flag(&mut self, flag: &str, value: f32) {
        let cur_val = match self.get_custom_flag(flag) {
            None => 0.0,
            Some(val_str) => val_str.parse::<f32>().unwrap_or(0.0),
        };
        self.set_custom_flag(flag, &(cur_val + value).to_string());
    }

    pub fn get_num_flag(&self, flag: &str) -> f32 {
        match self.get_custom_flag(flag) {
            None => 0.0,
            Some(val_str) => val_str.parse::<f32>().unwrap_or(0.0),
        }
    }

    pub fn has_custom_flag(&self, flag: &str) -> bool {
        self.custom_flags.contains_key(flag)
    }

    pub fn clear_texture_cache(&mut self) {
        self.texture_cache_slot = None;
    }

    pub fn ai_group(&self) -> Option<usize> {
        match self.ai_state {
            AIState::Player { .. } => None,
            AIState::AI { group, .. } => group,
        }
    }

    pub fn set_ai_active(&mut self, active: bool) {
        match self.ai_state {
            AIState::Player { .. } => (),
            AIState::AI { group, .. } => {
                self.ai_state = AIState::AI { group, active };
            }
        }
    }

    pub fn is_ai_active(&self) -> bool {
        match self.ai_state {
            AIState::Player { .. } => false,
            AIState::AI { active, .. } => active,
        }
    }

    pub fn show_portrait(&self) -> bool {
        match self.ai_state {
            AIState::Player { show_portrait, .. } => show_portrait,
            AIState::AI { .. } => false,
        }
    }

    pub fn add_to_party(&mut self, show_portrait: bool) {
        let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
        self.ai_state = AIState::Player {
            vis: vec![false; dim],
            show_portrait,
        };
    }

    pub fn remove_from_party(&mut self) {
        self.ai_state = AIState::AI {
            group: None,
            active: false,
        };
    }

    pub fn is_party_member(&self) -> bool {
        match self.ai_state {
            AIState::Player { .. } => true,
            AIState::AI { .. } => false,
        }
    }

    pub fn clear_pc_vis(&mut self) {
        match self.ai_state {
            AIState::Player { ref mut vis, .. } => unsafe {
                ptr::write_bytes(vis.as_mut_ptr(), 0, vis.len());
            },
            _ => panic!(),
        }
    }

    pub fn pc_vis_mut(&mut self) -> &mut Vec<bool> {
        match self.ai_state {
            AIState::Player { ref mut vis, .. } => vis,
            AIState::AI { .. } => panic!(),
        }
    }

    pub fn pc_vis(&self) -> &Vec<bool> {
        match self.ai_state {
            AIState::Player { ref vis, .. } => vis,
            AIState::AI { .. } => panic!(),
        }
    }

    pub fn explore_self_location(&self) {
        let area = GameState::get_area_state(&self.location.area_id).unwrap();
        let is_current = Rc::ptr_eq(&GameState::area_state(), &area);
        let mut changed = false;
        let mut area = area.borrow_mut();
        let width = area.area.width;

        for y in -2..self.size.height {
            for x in -1..self.size.width + 1 {
                let p = Point::new(x + self.location.x, y + self.location.y);
                let index = (p.x + p.y * width) as usize;
                if !area.pc_explored[index] {
                    changed = true;
                    area.pc_explored[index] = true;
                }
            }
        }

        if is_current && changed {
            area.pc_vis_full_redraw();
        }
    }

    pub fn is_hostile(&self, other: &EntityState) -> bool {
        let self_faction = self.actor.faction();
        let other_faction = other.actor.faction();

        self_faction.is_hostile(other_faction)
    }

    pub fn is_friendly(&self, other: &EntityState) -> bool {
        let self_faction = self.actor.faction();
        let other_faction = other.actor.faction();

        self_faction.is_friendly(other_faction)
    }

    pub(crate) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    pub fn swap_weapon_set(entity: &Rc<RefCell<EntityState>>) {
        if !entity.borrow_mut().actor.do_swap_weapons() {
            return;
        }

        if entity.borrow().is_party_member() && GameState::is_combat_active() {
            let area = GameState::area_state();
            let mut area = area.borrow_mut();
            area.range_indicators().remove_attack();
            area.range_indicators().add_attack(entity);
        }

        let mgr = GameState::turn_manager();
        let cbs = entity.borrow().callbacks(&mgr.borrow());
        cbs.iter().for_each(|cb| cb.on_held_changed());
    }

    /// Returns true if this entity has enough AP to move at least 1 square,
    /// false otherwise
    pub fn can_move(&self) -> bool {
        if self.actor.stats.move_disabled {
            return false;
        }

        self.actor.ap() >= self.actor.get_move_ap_cost(1)
    }

    /// Returns true if this entity can attack the specified target with its
    /// current weapon, without moving
    pub fn can_attack(&self, target: &EntityState) -> bool {
        if self.actor.stats.attack_disabled {
            return false;
        }

        if !self.actor.has_ap_to_attack() {
            return false;
        }

        is_within_attack_dist(self, target)
    }

    pub fn attack(
        entity: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
        callback: Option<Box<dyn ScriptCallback>>,
        remove_ap: bool,
    ) {
        let time = Config::animation_base_time_millis();
        let cbs: Vec<Box<dyn ScriptCallback>> = callback.into_iter().collect();
        if entity.borrow().actor.stats.attack_is_melee() {
            let anim = animation::melee_attack_animation::new(
                entity,
                target,
                time * 5,
                cbs,
                Box::new(weapon_attack),
            );
            GameState::add_animation(anim);
        } else if entity.borrow().actor.stats.attack_is_ranged() {
            let anim = animation::ranged_attack_animation::new(entity, target, cbs, time);
            GameState::add_animation(anim);
        }

        if remove_ap {
            let attack_ap = entity.borrow().actor.stats.attack_cost;
            entity.borrow_mut().actor.remove_ap(attack_ap as u32);
        }

        entity.borrow().explore_self_location();
    }

    pub fn add_xp(&mut self, xp: u32) {
        self.actor.add_xp(xp);
    }

    pub fn remove_hp(
        entity: &Rc<RefCell<EntityState>>,
        attacker: &Rc<RefCell<EntityState>>,
        hit_kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
    ) {
        let hp_amount = damage.iter().map(|(_, amount)| amount).sum();
        entity.borrow_mut().actor.remove_hp(hp_amount);

        let targets = ScriptEntitySet::from_pair(entity, attacker);

        let mgr = GameState::turn_manager();
        let cbs = entity.borrow().callbacks(&mgr.borrow());
        info!("Got {} cbs for {}", cbs.len(), entity.borrow().unique_id());
        cbs.iter()
            .for_each(|cb| cb.on_damaged(&targets, hit_kind, damage.clone()));

        let hp = entity.borrow().actor.hp();
        if hp <= 0 {
            debug!(
                "Entity '{}' has zero hit points.  Playing death animation",
                entity.borrow().actor.actor.name
            );
            let anim = Anim::new_entity_death(entity);
            GameState::add_animation(anim);
        } else {
            GameState::create_damage_animation(entity);
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32, squares: u32) -> bool {
        trace!("Move to {},{}", x, y);
        if !self.location.coords_valid(x, y) {
            return false;
        }
        if !self
            .location
            .coords_valid(x + self.size.width - 1, y + self.size.height - 1)
        {
            return false;
        }

        if x == self.location.x && y == self.location.y {
            return false;
        }

        let mgr = GameState::turn_manager();
        if mgr.borrow().is_combat_active() && squares > 0 {
            let ap_cost = self.actor.get_move_ap_cost(squares);
            if self.actor.ap() < ap_cost {
                return false;
            }
            self.actor.remove_ap(ap_cost);
        }

        self.location.move_to(x, y);
        self.listeners.notify(self);
        true
    }

    pub fn size(&self) -> &str {
        &self.size.id
    }

    pub fn relative_points(&self) -> ObjectSizeIterator {
        self.size.relative_points()
    }

    pub fn location_points(&self) -> ObjectSizeIterator {
        self.size.points(self.location.x, self.location.y)
    }

    pub fn points(&self, x: i32, y: i32) -> ObjectSizeIterator {
        self.size.points(x, y)
    }

    pub fn draw_no_pos(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        alpha: f32,
    ) {
        let a = self.color.a * alpha;
        let color = Color::new(self.color.r, self.color.g, self.color.b, a);
        if let Some(ref slot) = self.texture_cache_slot {
            let slot_loc = Slot {
                x: offset.x,
                y: offset.y,
            };
            slot.draw(
                renderer,
                slot_loc,
                Offset::default(),
                scale,
                color,
                self.color_sec,
            );
        }
    }
}

pub trait AreaDrawable {
    fn cache(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        texture_cache: &mut EntityTextureCache,
    );

    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        scale: Scale,
        x: f32,
        y: f32,
        millis: u32,
        color: Color,
    );

    fn size(&self) -> Size;

    fn location(&self) -> &Location;

    fn aerial(&self) -> bool;
}

impl AreaDrawable for EntityState {
    fn cache(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        texture_cache: &mut EntityTextureCache,
    ) {
        if self.texture_cache_slot.is_none() {
            self.texture_cache_slot = Some(texture_cache.add_entity(self, renderer));
            self.actor.check_texture_cache_invalid();
        }

        if self.actor.check_texture_cache_invalid() {
            let slot = &self.texture_cache_slot.as_ref().unwrap();
            slot.redraw_entity(self, renderer);
        }
    }

    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        scale: Scale,
        x: f32,
        y: f32,
        _millis: u32,
        color: Color,
    ) {
        // don't draw invisible hostiles
        if self.actor.stats.hidden {
            match self.actor.faction() {
                Faction::Hostile => return,
                Faction::Neutral => (),
                Faction::Friendly => (),
            }
        }

        let offset_x = (self.scale - 1.0) * self.size.width as f32 / 2.0;
        let offset_y = (self.scale - 1.0) * self.size.height as f32 / 2.0;
        let x = x + self.location.x as f32 + self.sub_pos.0;
        let y = y + self.location.y as f32 + self.sub_pos.1;

        let color = Color::new(
            self.color.r * color.r,
            self.color.g * color.g,
            self.color.b * color.b,
            self.color.a * color.a,
        );
        let offset = Offset {
            x: offset_x,
            y: offset_y,
        };
        let slot_loc = Slot { x, y };
        if let Some(ref slot) = self.texture_cache_slot {
            slot.draw(renderer, slot_loc, offset, scale, color, self.color_sec);
        }
    }

    fn size(&self) -> Size {
        Size { width: self.size.width, height: self.size.height }
    }

    fn location(&self) -> &Location {
        &self.location
    }

    fn aerial(&self) -> bool {
        false
    }
}
