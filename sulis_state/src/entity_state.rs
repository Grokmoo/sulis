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

use animation::{Animation, MeleeAttackAnimation, RangedAttackAnimation};
use sulis_core::io::GraphicsRenderer;
use sulis_module::{Actor, ObjectSize, ObjectSizeIterator, Module};
use sulis_module::area::Transition;
use {ActorState, AreaState, ChangeListenerList, GameState, has_visibility, Location, PropState, ScriptCallback};

pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<ObjectSize>,
    pub index: usize, // index in vec of the owning area state
    pub sub_pos: (f32, f32),
    pub listeners: ChangeListenerList<EntityState>,

    ai_group: Option<usize>,
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
                         index: usize, is_pc: bool, ai_group: Option<usize>) -> EntityState {
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
            ai_group,
        }
    }

    pub fn ai_group(&self) -> Option<usize> {
        self.ai_group
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
    pub fn can_attack(&self, target: &Rc<RefCell<EntityState>>, area: &Rc<Area>) -> bool {
        let dist = self.dist_to_entity(target);

        if !self.actor.can_weapon_attack(target, dist) { return false; }

        self.has_visibility(target, area)
    }

    pub fn attack(entity: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>,
                  callback: Option<Box<ScriptCallback>>) {
        let base_time = CONFIG.display.animation_base_time_millis;
        if entity.borrow().actor.stats.attack_is_melee() {
            let mut anim = MeleeAttackAnimation::new(entity, target, base_time * 5, Box::new(|att, def| {
                ActorState::weapon_attack(&att, &def)
            }));
            anim.set_callback(callback);
            GameState::add_animation(Box::new(anim));
        } else if entity.borrow().actor.stats.attack_is_ranged() {
            let mut anim = RangedAttackAnimation::new(entity, target, base_time);
            anim.set_callback(callback);
            GameState::add_animation(Box::new(anim));
        }

        entity.borrow_mut().actor.remove_ap(Module::rules().attack_ap);
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

    pub fn move_to(&mut self, area_state: &mut AreaState, x: i32, y: i32, squares: u32) -> bool {
        trace!("Move to {},{}", x, y);
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.size.width - 1, y + self.size.height - 1) {
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

    pub fn dist(&self, to_x: i32, to_y: i32, to_width: i32, to_height: i32) -> f32 {
        let self_half_size = self.size.diagonal as f32 / 2.0;
        let other_half_width = to_width as f32 / 2.0;
        let other_half_height = to_height as f32 / 2.0;
        let from_x = self.location.x as f32 + self_half_size;
        let from_y = self.location.y as f32 + self_half_size;
        let to_x = to_x as f32 + other_half_width;
        let to_y = to_y as f32 + other_half_height;

        ((from_x - to_x) * (from_x - to_x) + (from_y - to_y) * (from_y - to_y)).sqrt()
            - self_half_size - (other_half_width + other_half_height) / 2.0
    }

    pub fn dist_to_entity(&self, other: &Rc<RefCell<EntityState>>) -> f32 {
        let value = self.dist(other.borrow().location.x, other.borrow().location.y,
            other.borrow().size.width, other.borrow().size.height);

        trace!("Computed distance from '{}' at {:?} to '{}' at {:?} = {}", self.actor.actor.name,
               self.location, other.borrow().actor.actor.name, other.borrow().location, value);

        value
    }

    pub fn dist_to_transition(&self, other: &Transition) -> f32 {
        let value = self.dist(other.from.x, other.from.y, other.size.width, other.size.height);

        trace!("Computed distance from '{}' at {:?} to transition at {:?} = {}",
               self.actor.actor.name, self.location, other.from, value);

        value
    }

    pub fn dist_to_prop(&self, other: &PropState) -> f32 {
        self.dist(other.location.x, other.location.y,
                  other.prop.size.width, other.prop.size.height)
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
}

pub trait AreaDrawable {
    fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32, x: f32, y: f32,
            millis: u32);

    fn location(&self) -> &Location;
}

impl AreaDrawable for EntityState {
    fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32, x: f32, y: f32,
            millis: u32) {
        let x = x + self.location.x as f32 + self.sub_pos.0;
        let y = y + self.location.y as f32 + self.sub_pos.1;
        self.actor.draw_graphics_mode(renderer, scale_x, scale_y, x, y, millis);
    }

    fn location(&self) -> &Location {
        &self.location
    }
}
