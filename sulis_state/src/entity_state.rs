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
use std::cell::RefCell;

use sulis_core::config::CONFIG;
use sulis_module::Area;

use animation::{MeleeAttackAnimation, RangedAttackAnimation};
use sulis_module::{Actor, EntitySize, EntitySizeIterator};
use sulis_module::area::Transition;
use {ActorState, AreaState, ChangeListenerList, GameState, has_visibility, Location};

pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<EntitySize>,
    pub index: usize, // index in vec of the owning area state
    pub sub_pos: (f32, f32),
    pub listeners: ChangeListenerList<EntityState>,

    is_pc: bool,
    ai_active: bool,
    marked_for_removal: bool,
}

impl PartialEq for EntityState {
    fn eq(&self, other: &EntityState) -> bool {
        self.location.area_id == other.location.area_id &&
            self.index == other.index
    }
}

impl EntityState {
    pub(crate) fn new(actor: Rc<Actor>, location: Location,
                         index: usize, is_pc: bool) -> EntityState {
        debug!("Creating new entity state for {}", actor.id);
        let size = Rc::clone(&actor.race.size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state,
            location,
            sub_pos: (0.0, 0.0),
            size,
            index,
            listeners: ChangeListenerList::default(),
            is_pc,
            ai_active: false,
            marked_for_removal: false,
        }
    }

    pub fn set_ai_active(&mut self) {
        if self.is_pc() { return; }

        self.ai_active = true;
    }

    pub fn is_ai_active(&self) -> bool {
        self.ai_active
    }

    pub fn is_pc(&self) -> bool {
        self.is_pc
    }

    pub (crate) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    /// Returns true if this entity has enough AP to move at least 1 square,
    /// false otherwise
    pub fn can_move(&self) -> bool {
        self.actor.ap() >= self.actor.get_move_ap_cost(1)
    }

    /// Returns true if this entity can reach the specified target with its
    /// current weapon, without moving, false otherwise
    pub fn can_reach(&self, target: &Rc<RefCell<EntityState>>) -> bool {
        self.actor.can_reach(self.dist_to_entity(target))
    }

    /// Returns true if this entity can attack the specified target with its
    /// current weapon, without moving
    pub fn can_attack(&self, target: &Rc<RefCell<EntityState>>) -> bool {
        let dist = self.dist_to_entity(target);

        self.actor.can_attack(target, dist)
    }

    pub fn attack(entity: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {
        if entity.borrow().actor.stats.attack_is_melee() {
            let anim = MeleeAttackAnimation::new(entity, target,
                                                 CONFIG.display.animation_base_time_millis * 5);
            GameState::add_animation(Box::new(anim));
        } else if entity.borrow().actor.stats.attack_is_ranged() {
            let anim = RangedAttackAnimation::new(entity, target,
                                                  CONFIG.display.animation_base_time_millis);
            GameState::add_animation(Box::new(anim));
        }
    }

    pub fn remove_hp(&mut self, hp: u32) {
        self.actor.remove_hp(hp);

        if self.actor.hp() <= 0 {
            debug!("Entity '{}' has zero hit points.  Marked to remove.", self.actor.actor.name);
            self.marked_for_removal = true;
        }
    }

    pub fn move_to(&mut self, area_state: &mut AreaState, x: i32, y: i32, squares: u32) -> bool {
        trace!("Move to {},{}", x, y);
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.size() - 1, y + self.size() - 1) {
            return false;
        }

        if x == self.location.x && y == self.location.y {
            return false;
        }

        if area_state.turn_timer.is_active() {
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

    pub fn has_visibility(&self, other: &Rc<RefCell<EntityState>>, area: &Rc<Area>) -> bool {
        has_visibility(area, &self, &other.borrow())
    }

    fn dist(&self, to_x: i32, to_y: i32, to_size: i32) -> f32 {
        let self_half_size = self.size() as f32 / 2.0;
        let other_half_size = to_size as f32 / 2.0;
        let from_x = self.location.x as f32 + self_half_size;
        let from_y = self.location.y as f32 + self_half_size;
        let to_x = to_x as f32 + other_half_size;
        let to_y = to_y as f32 + other_half_size;

        ((from_x - to_x) * (from_x - to_x) + (from_y - to_y) * (from_y - to_y)).sqrt()
            - self_half_size - other_half_size
    }

    pub fn dist_to_entity(&self, other: &Rc<RefCell<EntityState>>) -> f32 {
        let value = self.dist(other.borrow().location.x, other.borrow().location.y, other.borrow().size());

        trace!("Computed distance from '{}' at {:?} to '{}' at {:?} = {}", self.actor.actor.name,
               self.location, other.borrow().actor.actor.name, other.borrow().location, value);

        value
    }

    pub fn dist_to_transition(&self, other: &Transition) -> f32 {
        let value = self.dist(other.from.x, other.from.y, other.size.width);

        trace!("Computed distance from '{}' at {:?} to transition at {:?} = {}",
               self.actor.actor.name, self.location, other.from, value);

        value
    }

    pub fn size(&self) -> i32 {
        self.size.size
    }

    pub fn relative_points(&self) -> EntitySizeIterator {
        self.size.relative_points()
    }

    pub fn location_points(&self) -> EntitySizeIterator {
        self.size.points(self.location.x, self.location.y)
    }

    pub fn points(&self, x: i32, y: i32) -> EntitySizeIterator {
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
}

