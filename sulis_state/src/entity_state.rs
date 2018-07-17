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
use std::ptr;
use std::slice::Iter;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use sulis_core::config::CONFIG;

use animation::{self};
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{color, Color};
use sulis_core::util::{Point, invalid_data_error};
use sulis_module::{actor::Faction, Actor, ObjectSize, ObjectSizeIterator, Module};
use sulis_module::area::{MAX_AREA_SIZE, Transition};
use {ActorState, AreaState, ChangeListenerList, EntityTextureCache, EntityTextureSlot,
    GameState, Location, PropState, ScriptCallback};
use save_state::EntitySaveState;

enum AIState {
    Player {
        vis: Vec<bool>,
    },
    AI {
        group: Option<usize>,
        active: bool,
    },
}

pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<ObjectSize>,
    pub index: usize, // index in vec of the owning area state
    pub sub_pos: (f32, f32),
    pub color: Color,
    pub color_sec: Color,
    pub listeners: ChangeListenerList<EntityState>,

    ai_state: AIState,
    marked_for_removal: bool,
    texture_cache_slot: Option<EntityTextureSlot>,

    custom_flags: Vec<String>,
}

impl PartialEq for EntityState {
    fn eq(&self, other: &EntityState) -> bool {
        self.location.area_id == other.location.area_id &&
            self.index == other.index
    }
}

impl EntityState {
    pub(crate) fn load(save: EntitySaveState,
                       areas: &HashMap<String, Rc<RefCell<AreaState>>>) -> Result<EntityState, Error> {
        let ai_state = match save.actor_base.as_ref() {
            None => {
                AIState::AI {
                    group: save.ai_group,
                    active: save.ai_active,
                }
            }, Some(_) => {
                let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
                AIState::Player {
                    vis: vec![false; dim],
                }
            },
        };

        let area = match areas.get(&save.location.area) {
            None => invalid_data_error(&format!("Invalid area '{}' for entity", save.location.area)),
            Some(area) => Ok(area),
        }?;

        let location = Location::new(save.location.x, save.location.y, &area.borrow().area);

        let size = match Module::object_size(&save.size) {
            None => invalid_data_error(&format!("Invalid size '{}' for actor", save.size)),
            Some(size) => Ok(size),
        }?;

        let actor = ActorState::load(save.actor, save.actor_base)?;

        Ok(EntityState {
            actor,
            location,
            size,
            index: save.index,
            sub_pos: (0.0, 0.0),
            color: color::WHITE,
            color_sec: Color::new(0.0, 0.0, 0.0, 0.0),
            listeners: ChangeListenerList::default(),
            ai_state,
            marked_for_removal: false,
            texture_cache_slot: None,
            custom_flags: save.custom_flags,
        })
    }

    pub(crate) fn new(actor: Rc<Actor>, location: Location,
                      index: usize, is_pc: bool, ai_group: Option<usize>) -> EntityState {
        let ai_state = if is_pc {
            let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
            AIState::Player {
                vis: vec![false; dim],
            }
        } else {
            AIState::AI {
                group: ai_group,
                active: false,
            }
        };

        debug!("Creating new entity state for {}", actor.id);
        let size = Rc::clone(&actor.race.size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state,
            location,
            sub_pos: (0.0, 0.0),
            color: color::WHITE,
            color_sec: Color::new(0.0, 0.0, 0.0, 0.0),
            size,
            index,
            listeners: ChangeListenerList::default(),
            marked_for_removal: false,
            ai_state,
            texture_cache_slot: None,
            custom_flags: Vec::new(),
        }
    }

    pub fn callbacks(&self) -> Vec<Rc<ScriptCallback>> {
        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        self.actor.effects_iter().flat_map(|index| {
            mgr.effect(*index).callbacks()
        }).collect()
    }

    pub fn custom_flags<'a>(&'a self) -> Iter<'a, String> {
        self.custom_flags.iter()
    }

    pub fn set_custom_flag(&mut self, flag: &str) {
        self.custom_flags.push(flag.to_string());
        self.custom_flags.sort();
        self.custom_flags.dedup();
    }

    pub fn has_custom_flag(&self, flag: &str) -> bool {
        self.custom_flags.iter().any(|x| x == flag)
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
                self.ai_state = AIState::AI {
                    group,
                    active,
                };
            }
        }
    }

    pub fn is_ai_active(&self) -> bool {
        match self.ai_state {
            AIState::Player { .. } => false,
            AIState::AI { active, .. } => active,
        }
    }

    pub fn set_party_member(&mut self, member: bool) {
        if member {
            let dim = (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize;
            self.ai_state = AIState::Player {
                vis: vec![false; dim],
            };
        } else {
            self.ai_state = AIState::AI {
                group: None,
                active: false,
            }
        }
    }

    pub fn is_party_member(&self) -> bool {
        match self.ai_state {
            AIState::Player { .. } => true,
            AIState::AI { .. } => false,
        }
    }

    pub fn clear_pc_vis(&mut self) {
        match self.ai_state {
            AIState::Player { ref mut vis, .. } => {
                unsafe {
                    ptr::write_bytes(vis.as_mut_ptr(), 0, vis.len());
                }
            },
            _ => panic!(),
        }
    }

    pub fn pc_vis_mut<'a>(&'a mut self) -> &'a mut Vec<bool> {
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

    pub fn is_hostile(&self, other: &Rc<RefCell<EntityState>>) -> bool {
        self.actor.actor.faction != other.borrow().actor.actor.faction
    }

    pub (crate) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    /// Returns true if this entity has enough AP to move at least 1 square,
    /// false otherwise
    pub fn can_move(&self) -> bool {
        if self.actor.stats.move_disabled { return false; }

        self.actor.ap() >= self.actor.get_move_ap_cost(1)
    }

    /// Returns true if this entity can reach the specified target with its
    /// current weapon, without moving, false otherwise
    pub fn can_reach(&self, target: &Rc<RefCell<EntityState>>) -> bool {
        let dist = self.dist_to_entity(target);
        let area = GameState::area_state();
        let vis_dist = area.borrow().area.vis_dist as f32;

        if dist > vis_dist {
            false
        } else {
            self.actor.can_reach(self.dist_to_entity(target))
        }
    }

    /// Returns true if this entity can attack the specified target with its
    /// current weapon, without moving
    pub fn can_attack(&self, target: &Rc<RefCell<EntityState>>, area_state: &AreaState) -> bool {
        if self.actor.stats.attack_disabled { return false; }

        let dist = self.dist_to_entity(target);

        if !self.actor.can_weapon_attack(target, dist) { return false; }

        area_state.has_visibility(&self, &target.borrow())
    }

    pub fn attack(entity: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                  callback: Option<Box<ScriptCallback>>) {
        let time = CONFIG.display.animation_base_time_millis;
        let cbs: Vec<Box<ScriptCallback>> = callback.into_iter().collect();
        if entity.borrow().actor.stats.attack_is_melee() {
            let anim = animation::melee_attack_animation::new(entity, target, time * 5, cbs, Box::new(|a, d| {
                ActorState::weapon_attack(&a, &d)
            }));
            GameState::add_animation(anim);
        } else if entity.borrow().actor.stats.attack_is_ranged() {
            let anim = animation::ranged_attack_animation::new(entity, target, cbs, time);
            GameState::add_animation(anim);
        }

        let attack_ap = entity.borrow().actor.stats.attack_cost;
        entity.borrow_mut().actor.remove_ap(attack_ap as u32);
    }

    pub fn add_xp(&mut self, xp: u32) {
        self.actor.add_xp(xp);
    }

    pub fn remove_hp(&mut self, hp: u32) {
        self.actor.remove_hp(hp);

        if self.actor.hp() <= 0 {
            debug!("Entity '{}' has zero hit points.  Marked to remove.", self.actor.actor.name);
            self.marked_for_removal = true;
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32, squares: u32) -> bool {
        trace!("Move to {},{}", x, y);
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.size.width - 1, y + self.size.height - 1) {
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
        self.listeners.notify(&self);
        true
    }

    pub fn dist_to_point(&self, pos: Point) -> f32 {
        let x1 = self.location.x as f32 + (self.size.width / 2) as f32;
        let y1 = self.location.y as f32 + (self.size.height / 2) as f32;
        let x2 = pos.x as f32;
        let y2 = pos.y as f32;

        let dist = ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
        dist - self.size.diagonal / 2.0 - (2.0 as f32).sqrt() / 2.0
    }

    pub fn dist(&self, pos: Point, size: &Rc<ObjectSize>) -> f32 {
        let x1 = self.location.x as f32 + (self.size.width / 2) as f32;
        let y1 = self.location.y as f32 + (self.size.height / 2) as f32;

        let x2 = pos.x as f32 + size.width as f32 / 2.0;
        let y2 = pos.y as f32 + size.height as f32 / 2.0;

        let dist = ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
        dist - self.size.diagonal / 2.0 - size.diagonal / 2.0
    }

    pub fn dist_to_entity(&self, other: &Rc<RefCell<EntityState>>) -> f32 {
        let value = self.dist(other.borrow().location.to_point(), &other.borrow().size);

        trace!("Computed distance from '{}' at {:?} to '{}' at {:?} = {}", self.actor.actor.name,
               self.location, other.borrow().actor.actor.name, other.borrow().location, value);

        value
    }

    pub fn dist_to_transition(&self, other: &Transition) -> f32 {
        let value = self.dist(other.from, &other.size);

        trace!("Computed distance from '{}' at {:?} to transition at {:?} = {}",
               self.actor.actor.name, self.location, other.from, value);

        value
    }

    pub fn dist_to_prop(&self, other: &PropState) -> f32 {
        self.dist(other.location.to_point(), &other.prop.size)
    }

    pub fn center_x_f32(&self) -> f32 {
        self.location.x as f32 + self.size.width as f32 / 2.0
    }

    pub fn center_y_f32(&self) -> f32 {
        self.location.y as f32 + self.size.height as f32 / 2.0
    }

    pub fn center_x(&self) -> i32 {
        self.location.x + self.size.width / 2
    }

    pub fn center_y(&self) -> i32 {
        self.location.y + self.size.height / 2
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

    /// Returns true if `e1` and `e2` refer to the same EntityState, false
    /// otherwise
    pub fn equals(e1: &Rc<RefCell<EntityState>>, e2: &Rc<RefCell<EntityState>>) -> bool {
        if e1.borrow().location != e2.borrow().location {
            return false;
        }

        e1.borrow().index == e2.borrow().index
    }

    pub fn draw_no_pos(&self, renderer: &mut GraphicsRenderer,
                   scale_x: f32, scale_y: f32, x: f32, y: f32, alpha: f32) {
        let a = self.color.a * alpha;
        let color = Color::new(self.color.r, self.color.g, self.color.b, a);
        if let Some(ref slot) = self.texture_cache_slot {
            slot.draw(renderer, x, y, scale_x, scale_y, color, self.color_sec);
        }
    }
}

pub trait AreaDrawable {
    fn cache(&mut self, renderer: &mut GraphicsRenderer, texture_cache: &mut EntityTextureCache);

    fn draw(&self, renderer: &mut GraphicsRenderer,
            scale_x: f32, scale_y: f32, x: f32, y: f32, millis: u32, alpha: f32);

    fn location(&self) -> &Location;
}

impl AreaDrawable for EntityState {
    fn cache(&mut self, renderer: &mut GraphicsRenderer, texture_cache: &mut EntityTextureCache) {
        if self.texture_cache_slot.is_none() {
            self.texture_cache_slot = Some(texture_cache.add_entity(&self, renderer));
            self.actor.check_texture_cache_invalid();
        }

        if self.actor.check_texture_cache_invalid() {
            let slot = &self.texture_cache_slot.as_ref().unwrap();
            slot.redraw_entity(&self, renderer);
        }
    }

    fn draw(&self, renderer: &mut GraphicsRenderer,
            scale_x: f32, scale_y: f32, x: f32, y: f32, _millis: u32, alpha: f32) {
        // don't draw invisible hostiles
        if self.actor.stats.hidden {
            match self.actor.actor.faction {
                Faction::Hostile => return,
                Faction::Friendly => (),
            }
        }

        let x = x + self.location.x as f32 + self.sub_pos.0;
        let y = y + self.location.y as f32 + self.sub_pos.1;

        let a = self.color.a * alpha;
        let color = Color::new(self.color.r, self.color.g, self.color.b, a);
        if let Some(ref slot) = self.texture_cache_slot {
            slot.draw(renderer, x, y, scale_x, scale_y, color, self.color_sec);
        }
    }

    fn location(&self) -> &Location {
        &self.location
    }
}
