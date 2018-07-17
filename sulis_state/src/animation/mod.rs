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

use std::time::{Instant, Duration};
use std::cmp;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

mod entity_color_animation;

mod entity_subpos_animation;

pub mod melee_attack_animation;

pub mod move_animation;

pub mod particle_generator;

pub mod ranged_attack_animation;

use sulis_core::{io::GraphicsRenderer, ui::Widget, util::{self, ExtInt}};

use {ChangeListener, Effect, EntityState, ScriptCallback};
use self::particle_generator::Param;
use self::melee_attack_animation::MeleeAttackAnimModel;
use self::ranged_attack_animation::RangedAttackAnimModel;
use self::move_animation::MoveAnimModel;
use self::particle_generator::{GeneratorModel, GeneratorState};

pub struct AnimState {
    no_draw_anims: Vec<Anim>,
    below_anims: Vec<Anim>,
    above_anims: Vec<Anim>,
}

impl AnimState {
    pub fn new() -> AnimState {
        AnimState {
            no_draw_anims: Vec::new(),
            below_anims: Vec::new(),
            above_anims: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.no_draw_anims.clear();
        self.below_anims.clear();
        self.above_anims.clear();
    }

    pub fn update(&mut self, to_add: Vec<Anim>, root: &Rc<RefCell<Widget>>) {
        for anim in to_add {
            use self::AnimKind::*;
            match anim.kind {
                RangedAttack { .. } =>
                    self.above_anims.push(anim),
                ParticleGenerator { .. } => // TODO support below anims
                    self.above_anims.push(anim),
                _ =>
                    self.no_draw_anims.push(anim),
            }
        }

        AnimState::update_vec(&mut self.no_draw_anims, root);
        AnimState::update_vec(&mut self.below_anims, root);
        AnimState::update_vec(&mut self.above_anims, root);
    }

    pub fn draw_below_entities(&self, renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                scale_x: f32, scale_y: f32, millis: u32) {
        for anim in self.below_anims.iter() {
            anim.draw(renderer, offset_x, offset_y, scale_x, scale_y, millis);
        }
    }

    pub fn draw_above_entities(&self, renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                               scale_x: f32, scale_y: f32, millis: u32) {
        for anim in self.above_anims.iter() {
            anim.draw(renderer, offset_x, offset_y, scale_x, scale_y, millis);
        }
    }

    pub fn has_blocking_anims(&self, entity: &Rc<RefCell<EntityState>>) -> bool {
        AnimState::has_blocking_vec(&self.no_draw_anims, entity) ||
            AnimState::has_blocking_vec(&self.below_anims, entity) ||
            AnimState::has_blocking_vec(&self.above_anims, entity)
    }

    pub fn clear_blocking_anims(&mut self, entity: &Rc<RefCell<EntityState>>) {
        AnimState::clear_blocking_vec(&mut self.no_draw_anims, entity);
        AnimState::clear_blocking_vec(&mut self.below_anims, entity);
        AnimState::clear_blocking_vec(&mut self.above_anims, entity);
    }

    pub fn clear_all_blocking_anims(&mut self) {
        AnimState::clear_all_blocking_vec(&mut self.no_draw_anims);
        AnimState::clear_all_blocking_vec(&mut self.below_anims);
        AnimState::clear_all_blocking_vec(&mut self.above_anims);
    }

    fn has_blocking_vec(vec: &Vec<Anim>, entity: &Rc<RefCell<EntityState>>) -> bool {
        for anim in vec.iter() {
            if !anim.is_blocking() { continue; }
            if !Rc::ptr_eq(entity, anim.owner()) { continue; }

            return true;
        }
        false
    }

    fn clear_blocking_vec(vec: &mut Vec<Anim>, entity: &Rc<RefCell<EntityState>>) {
        for anim in vec.iter_mut() {
            if !anim.is_blocking() { continue; }
            if !Rc::ptr_eq(entity, anim.owner()) { continue; }

            anim.mark_for_removal();
        }
    }

    fn clear_all_blocking_vec(vec: &mut Vec<Anim>) {
        for anim in vec.iter_mut() {
            if anim.is_blocking() {
                anim.mark_for_removal();
            }
        }
    }

    fn update_vec(vec: &mut Vec<Anim>, root: &Rc<RefCell<Widget>>) {
        let mut i = 0;
        while i < vec.len() {
            let retain = vec[i].update(root);

            if retain { i += 1; }
            else {
                vec[i].cleanup_kind();
                vec[i].run_completion_callbacks();
                vec.remove(i);
            }
        }
    }
}

pub struct Anim {
    kind: AnimKind,
    start_time: Instant,
    duration_millis: ExtInt,
    marked_for_removal: Rc<Cell<bool>>,
    completion_callbacks: Vec<Box<ScriptCallback>>,
    update_callbacks: Vec<(u32, Box<ScriptCallback>)>, // sorted by the first field, time in secs
    owner: Rc<RefCell<EntityState>>,
}

pub (in animation) enum AnimKind {
    /// An animation that does nothing, but blocks the entity for a specified time
    Wait,

    /// An animation that changes the color of an entity while active
    EntityColor { color: [Param; 4], color_sec: [Param; 4] },

    /// An animation that changes the position of an entity while active
    EntitySubpos { x: Param, y: Param },

    /// An attack with a melee weapon
    MeleeAttack { model: MeleeAttackAnimModel },

    /// An attack with a ranged weapon
    RangedAttack { model: RangedAttackAnimModel },

    /// Movement of a single entity within an area
    Move { model: MoveAnimModel },

    /// A particle effect from a script - can also be used for simple
    /// single image animations
    ParticleGenerator { model: GeneratorModel, state: GeneratorState },
}

impl Anim {
    pub fn new_wait(owner: &Rc<RefCell<EntityState>>, duration_millis: u32) -> Anim {
        Anim::new(owner, ExtInt::Int(duration_millis), AnimKind::Wait)
    }

    pub fn new_entity_color(owner: &Rc<RefCell<EntityState>>, duration_millis: ExtInt,
                            color: [Param; 4], color_sec: [Param; 4]) -> Anim {
        Anim::new(owner, duration_millis, AnimKind::EntityColor { color, color_sec })
    }

    pub fn new_entity_subpos(owner: &Rc<RefCell<EntityState>>, duration_millis: ExtInt,
                             x: Param, y: Param) -> Anim {
        Anim::new(owner, duration_millis, AnimKind::EntitySubpos { x, y })
    }

    pub (in animation) fn new_melee_attack(attacker: &Rc<RefCell<EntityState>>, duration_millis: u32,
                            model: MeleeAttackAnimModel) -> Anim {
        Anim::new(attacker, ExtInt::Int(duration_millis), AnimKind::MeleeAttack { model })
    }

    pub (in animation) fn new_ranged_attack(attacker: &Rc<RefCell<EntityState>>, duration_millis: u32,
                                            model: RangedAttackAnimModel) -> Anim {
        Anim::new(attacker, ExtInt::Int(duration_millis), AnimKind::RangedAttack { model })
    }

    pub (in animation) fn new_move(mover: &Rc<RefCell<EntityState>>, duration_millis: u32,
                                   model: MoveAnimModel) -> Anim {
        Anim::new(mover, ExtInt::Int(duration_millis), AnimKind::Move { model })
    }

    pub (in animation) fn new_pgen(owner: &Rc<RefCell<EntityState>>, duration_millis: ExtInt,
                                   model: GeneratorModel, state: GeneratorState) -> Anim {
        Anim::new(owner, duration_millis, AnimKind::ParticleGenerator { model, state })
    }

    fn new(owner: &Rc<RefCell<EntityState>>, duration_millis: ExtInt,
                              kind: AnimKind) -> Anim {
        Anim {
            kind,
            start_time: Instant::now(),
            duration_millis,
            marked_for_removal: Rc::new(Cell::new(false)),
            completion_callbacks: Vec::new(),
            update_callbacks: Vec::new(),
            owner: Rc::clone(owner),
        }
    }

    pub fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        let millis = util::get_elapsed_millis(self.start_time.elapsed());

        self.update_kind(millis);

        if !self.update_callbacks.is_empty() {
            if millis > self.update_callbacks[0].0 {
                self.update_callbacks[0].1.on_anim_update();
                self.update_callbacks.remove(0);
            }
        }

        if (self.duration_millis.less_than(millis) && self.ok_to_remove()) || self.marked_for_removal.get() {
            false
        } else {
            true
        }
    }

    fn run_completion_callbacks(&mut self) {
        for cb in self.update_callbacks.iter() {
            cb.1.on_anim_update();
        }
        self.update_callbacks.clear();

        for cb in self.completion_callbacks.iter() {
            cb.on_anim_complete();
        }
        self.completion_callbacks.clear();
    }

    fn ok_to_remove(&self) -> bool {
        use self::AnimKind::*;
        match self.kind {
            MeleeAttack { ref model } => model.has_attacked,
            RangedAttack { ref model } => model.has_attacked,
            Move { ref model } => model.last_frame_index == model.path.len() as i32 - 1,
            _ => true,
        }
    }

    fn update_kind(&mut self, millis: u32) {
        let frac = millis as f32 / self.duration_millis.to_f32();
        use self::AnimKind::*;
        match self.kind {
            EntityColor { ref mut color, ref mut color_sec } =>
                entity_color_animation::update(color, color_sec, &self.owner, millis),
            EntitySubpos { ref mut x, ref mut y } =>
                entity_subpos_animation::update(x, y, &self.owner, millis),
            MeleeAttack { ref mut model } =>
                melee_attack_animation::update(&self.owner, model, frac),
            RangedAttack { ref mut model } =>
                ranged_attack_animation::update(&self.owner, model, frac),
            Move { ref mut model } =>
                move_animation::update(&self.owner, &self.marked_for_removal, model, millis),
            ParticleGenerator { ref mut model, ref mut state } =>
                particle_generator::update(&self.owner, model, state, &self.marked_for_removal, millis),
            _ => (),
        }
    }

    fn cleanup_kind(&mut self) {
        use self::AnimKind::*;
        match self.kind {
            EntityColor { .. } => entity_color_animation::cleanup(&self.owner),
            EntitySubpos { .. } => entity_subpos_animation::cleanup(&self.owner),
            MeleeAttack { .. } => melee_attack_animation::cleanup(&self.owner),
            Move { .. } => move_animation::cleanup(&self.owner),
            _ => (),
        }
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                          scale_x: f32, scale_y: f32, millis: u32) {
        use self::AnimKind::*;
        match self.kind {
            RangedAttack { ref model } =>
                ranged_attack_animation::draw(&model, renderer, offset_x, offset_y, scale_x, scale_y, millis),
            ParticleGenerator { ref state, ref model } =>
                particle_generator::draw(state, model, &self.owner, renderer,
                                         offset_x, offset_y, scale_x, scale_y, millis),
            _ => (),
        }
    }

    pub fn add_completion_callback(&mut self, callback: Box<ScriptCallback>) {
        self.completion_callbacks.push(callback);
    }

    pub fn add_update_callback(&mut self, callback: Box<ScriptCallback>, time_millis: u32) {
        self.update_callbacks.push((time_millis, callback));

        self.update_callbacks.sort_by(|a, b| {
            if a.0 < b.0 {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        });
    }

    pub fn add_removal_listener(&self, effect: &mut Effect) {
        let marked_for_removal = Rc::clone(&self.marked_for_removal);
        effect.removal_listeners.add(ChangeListener::new("anim", Box::new(move |_| {
            marked_for_removal.set(true);
        })));
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal.set(true);
    }

    pub fn owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }

    pub fn is_blocking(&self) -> bool {
        use self::AnimKind::*;
        match self.kind {
            EntityColor { .. } => false,
            EntitySubpos { .. } => true,
            MeleeAttack { .. } => true,
            RangedAttack { .. } => true,
            Move { .. } => true,
            Wait => true,
            ParticleGenerator { .. } => false,
        }
    }
}

/// Helper function to return the number of frames elapsed for the 'elapsed'
/// duration.  This is equal to the duration divided by 'frame_time'.  The
/// value is capped by 'max_index'
pub fn get_current_frame(elapsed: Duration, frame_time:
                         u32, max_index: usize) -> usize {
    let millis = util::get_elapsed_millis(elapsed);

    let frame_index = (millis / frame_time) as usize;

    cmp::min(frame_index, max_index)
}
