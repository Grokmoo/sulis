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

use std::cell::{Cell, RefCell};
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

mod anim_save_state;
pub use self::anim_save_state::AnimSaveState;

mod entity_color_animation;

mod entity_scale_animation;

mod entity_subpos_animation;

pub mod melee_attack_animation;

pub mod move_animation;

pub mod particle_generator;

pub mod ranged_attack_animation;

use self::melee_attack_animation::MeleeAttackAnimModel;
use self::move_animation::MoveAnimModel;
use self::particle_generator::Param;
use self::particle_generator::{GeneratorModel, GeneratorState};
use self::ranged_attack_animation::RangedAttackAnimModel;
use crate::{ChangeListener, Effect, EntityState, ScriptCallback};
use sulis_core::{
    image::Image,
    io::GraphicsRenderer,
    util::{self, ExtInt, Offset, Scale},
};
use sulis_module::ImageLayer;

pub struct AnimState {
    no_draw_anims: Vec<Anim>,
    below_anims: Vec<Anim>,
    above_anims: Vec<Anim>,
}

impl Default for AnimState {
    fn default() -> Self {
        Self::new()
    }
}

type BoxedCB = Box<dyn ScriptCallback>;

impl AnimState {
    pub const fn new() -> AnimState {
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

    /// Updates all animations based on the elapsed time.  Adds the specified animations.
    /// Returns two vecs, the first is the list of animations that should be `on_anim_update`,
    /// the second is the list that should be `on_anim_complete`
    #[must_use]
    pub fn update(&mut self, to_add: Vec<Anim>, elapsed: u32) -> (Vec<BoxedCB>, Vec<BoxedCB>) {
        let mut update_cbs = Vec::new();
        let mut complete_cbs = Vec::new();

        for anim in to_add {
            use self::AnimKind::*;
            let draw_above = match anim.kind {
                ParticleGenerator { ref model, .. } => model.draw_above_entities,
                _ => false,
            };

            match anim.kind {
                RangedAttack { .. } => self.above_anims.push(anim),
                ParticleGenerator { .. } => {
                    if draw_above {
                        self.above_anims.push(anim);
                    } else {
                        self.below_anims.push(anim);
                    }
                }
                Move { .. } => {
                    if anim.owner.borrow().is_party_member() {
                        self.below_anims.push(anim);
                    } else {
                        self.no_draw_anims.push(anim);
                    }
                }
                _ => self.no_draw_anims.push(anim),
            }
        }

        AnimState::update_vec(
            &mut self.no_draw_anims,
            elapsed,
            &mut update_cbs,
            &mut complete_cbs,
        );
        AnimState::update_vec(
            &mut self.below_anims,
            elapsed,
            &mut update_cbs,
            &mut complete_cbs,
        );
        AnimState::update_vec(
            &mut self.above_anims,
            elapsed,
            &mut update_cbs,
            &mut complete_cbs,
        );

        (update_cbs, complete_cbs)
    }

    pub fn save_anims(&self) -> Vec<AnimSaveState> {
        let mut anims = Vec::new();
        self.below_anims
            .iter()
            .for_each(|anim| anims.push(AnimSaveState::new(anim)));
        self.above_anims
            .iter()
            .for_each(|anim| anims.push(AnimSaveState::new(anim)));
        self.no_draw_anims
            .iter()
            .for_each(|anim| anims.push(AnimSaveState::new(anim)));

        anims
    }

    pub fn draw_below_entities(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        // TODO need to only draw animations that are in the current area
        for anim in self.below_anims.iter() {
            anim.draw(renderer, offset, scale, millis);
        }
    }

    pub fn draw_above_entities(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        // TODO only draw anims in the current area
        for anim in self.above_anims.iter() {
            anim.draw(renderer, offset, scale, millis);
        }
    }

    pub fn has_any_blocking_anims(&self) -> bool {
        AnimState::has_any_blocking_vec(&self.no_draw_anims)
            || AnimState::has_any_blocking_vec(&self.below_anims)
            || AnimState::has_any_blocking_vec(&self.above_anims)
    }

    pub fn anim_blocked_time(&self, entity: &Rc<RefCell<EntityState>>) -> ExtInt {
        let v1 = AnimState::blocked_time_vec(&self.no_draw_anims, entity);
        let v2 = AnimState::blocked_time_vec(&self.below_anims, entity);
        let v3 = AnimState::blocked_time_vec(&self.above_anims, entity);
        v1.max(v2).max(v3)
    }

    pub fn has_blocking_anims(&self, entity: &Rc<RefCell<EntityState>>) -> bool {
        AnimState::has_blocking_vec(&self.no_draw_anims, entity)
            || AnimState::has_blocking_vec(&self.below_anims, entity)
            || AnimState::has_blocking_vec(&self.above_anims, entity)
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

    fn has_any_blocking_vec(vec: &[Anim]) -> bool {
        for anim in vec.iter() {
            if !anim.is_blocking() {
                continue;
            }

            return true;
        }
        false
    }

    fn blocked_time_vec(vec: &[Anim], entity: &Rc<RefCell<EntityState>>) -> ExtInt {
        let mut time = ExtInt::Int(0);
        for anim in vec {
            if !anim.is_blocking() {
                continue;
            }
            if !Rc::ptr_eq(entity, anim.owner()) {
                continue;
            }

            let remaining = anim.duration_millis - anim.elapsed;
            time = time.max(remaining);
        }

        time
    }

    fn has_blocking_vec(vec: &[Anim], entity: &Rc<RefCell<EntityState>>) -> bool {
        for anim in vec.iter() {
            if !anim.is_blocking() {
                continue;
            }
            if !Rc::ptr_eq(entity, anim.owner()) {
                continue;
            }

            return true;
        }
        false
    }

    fn clear_blocking_vec(vec: &mut [Anim], entity: &Rc<RefCell<EntityState>>) {
        for anim in vec.iter_mut() {
            if !anim.is_blocking() {
                continue;
            }
            if !Rc::ptr_eq(entity, anim.owner()) {
                continue;
            }

            anim.mark_for_removal();
            // don't fire any more callbacks for removed anims
            anim.completion_callbacks.clear();
            anim.update_callbacks.clear();
        }
    }

    fn clear_all_blocking_vec(vec: &mut [Anim]) {
        for anim in vec.iter_mut() {
            if anim.is_blocking() {
                anim.mark_for_removal();
                // don't fire any more callbacks for removed anims
                anim.completion_callbacks.clear();
                anim.update_callbacks.clear();
            }
        }
    }

    fn update_vec(
        vec: &mut Vec<Anim>,
        elapsed: u32,
        update_cbs: &mut Vec<Box<dyn ScriptCallback>>,
        complete_cbs: &mut Vec<Box<dyn ScriptCallback>>,
    ) {
        let mut i = 0;
        while i < vec.len() {
            let retain = vec[i].update(elapsed);

            if retain {
                i += 1;
            } else {
                vec[i].cleanup_kind();
                vec[i].get_completion_callbacks(update_cbs, complete_cbs);
                vec.remove(i);
            }
        }
    }
}

pub struct Anim {
    pub(in crate::animation) kind: AnimKind,
    pub(in crate::animation) elapsed: u32,
    pub(in crate::animation) duration_millis: ExtInt,
    marked_for_removal: Rc<Cell<bool>>,
    completion_callbacks: Vec<Box<dyn ScriptCallback>>,
    update_callbacks: Vec<(u32, Box<dyn ScriptCallback>)>, // sorted by the first field, time in secs
    pub(in crate::animation) owner: Rc<RefCell<EntityState>>,
    pub(in crate::animation) removal_effect: Option<usize>, // used only for save/load purposes
}

pub(in crate::animation) enum AnimKind {
    /// An animation that does nothing, but blocks the entity for a specified time
    Wait,

    /// An animation that does nothing, and does not block
    NonBlockingWait,

    /// An animation that adds or overrides image layers on an entity
    /// while active
    EntityImageLayer {
        images: HashMap<ImageLayer, Rc<dyn Image>>,
    },

    /// An animation that changes the color of an entity while active
    EntityColor {
        color: [Param; 4],
        color_sec: [Param; 4],
    },

    /// An animation that changes the size of an entity while active
    EntityScale { scale: Param },

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
    ParticleGenerator {
        model: Box<GeneratorModel>,
        state: Box<GeneratorState>,
    },

    /// Animation triggered when an entity is killed
    EntityDeath {
        color: [Param; 4],
        color_sec: [Param; 4],
    },
}

impl Anim {
    pub fn new_non_blocking_wait(owner: &Rc<RefCell<EntityState>>, duration_millis: u32) -> Anim {
        Anim::new(
            owner,
            ExtInt::Int(duration_millis),
            AnimKind::NonBlockingWait,
        )
    }

    pub fn new_wait(owner: &Rc<RefCell<EntityState>>, duration_millis: u32) -> Anim {
        Anim::new(owner, ExtInt::Int(duration_millis), AnimKind::Wait)
    }

    pub fn new_entity_image_layer(
        owner: &Rc<RefCell<EntityState>>,
        duration_millis: ExtInt,
        images: HashMap<ImageLayer, Rc<dyn Image>>,
    ) -> Anim {
        Anim::new(
            owner,
            duration_millis,
            AnimKind::EntityImageLayer { images },
        )
    }

    pub fn new_entity_color(
        owner: &Rc<RefCell<EntityState>>,
        duration_millis: ExtInt,
        color: [Param; 4],
        color_sec: [Param; 4],
    ) -> Anim {
        Anim::new(
            owner,
            duration_millis,
            AnimKind::EntityColor { color, color_sec },
        )
    }

    pub fn new_entity_scale(
        owner: &Rc<RefCell<EntityState>>,
        duration_millis: ExtInt,
        scale: Param,
    ) -> Anim {
        Anim::new(owner, duration_millis, AnimKind::EntityScale { scale })
    }

    pub fn new_entity_subpos(
        owner: &Rc<RefCell<EntityState>>,
        duration_millis: ExtInt,
        x: Param,
        y: Param,
    ) -> Anim {
        Anim::new(owner, duration_millis, AnimKind::EntitySubpos { x, y })
    }

    pub(in crate::animation) fn new_melee_attack(
        attacker: &Rc<RefCell<EntityState>>,
        duration_millis: u32,
        model: MeleeAttackAnimModel,
    ) -> Anim {
        Anim::new(
            attacker,
            ExtInt::Int(duration_millis),
            AnimKind::MeleeAttack { model },
        )
    }

    pub(in crate::animation) fn new_ranged_attack(
        attacker: &Rc<RefCell<EntityState>>,
        duration_millis: u32,
        model: RangedAttackAnimModel,
    ) -> Anim {
        Anim::new(
            attacker,
            ExtInt::Int(duration_millis),
            AnimKind::RangedAttack { model },
        )
    }

    pub(in crate::animation) fn new_move(
        mover: &Rc<RefCell<EntityState>>,
        duration_millis: u32,
        model: MoveAnimModel,
    ) -> Anim {
        Anim::new(
            mover,
            ExtInt::Int(duration_millis),
            AnimKind::Move { model },
        )
    }

    pub(in crate::animation) fn new_pgen(
        owner: &Rc<RefCell<EntityState>>,
        duration_millis: ExtInt,
        model: GeneratorModel,
        state: GeneratorState,
    ) -> Anim {
        Anim::new(
            owner,
            duration_millis,
            AnimKind::ParticleGenerator {
                model: Box::new(model),
                state: Box::new(state),
            },
        )
    }

    pub fn new_entity_recover(owner: &Rc<RefCell<EntityState>>) -> Anim {
        let duration_millis = ExtInt::Int(800);
        let fixed = Param::fixed(1.0);
        let vel = Param::with_speed(1.0, -1.0);
        let color = [fixed, fixed, fixed, fixed];
        let color_sec = [vel, vel, vel, Param::fixed(0.0)];
        Anim::new(
            owner,
            duration_millis,
            AnimKind::EntityColor { color, color_sec },
        )
    }

    pub fn new_entity_death(owner: &Rc<RefCell<EntityState>>) -> Anim {
        let time = 800;
        let time_f32 = time as f32 / 1000.0;
        let duration_millis = ExtInt::Int(time);
        let alpha = Param::with_jerk(1.0, 0.0, 0.0, -1.0 / time_f32);
        let color = [
            Param::fixed(1.0),
            Param::fixed(0.0),
            Param::fixed(0.0),
            alpha,
        ];
        let red = Param::with_accel(0.0, 1.0 / time_f32, -1.0 / time_f32);
        let color_sec = [red, Param::fixed(0.0), Param::fixed(0.0), Param::fixed(0.0)];
        Anim::new(
            owner,
            duration_millis,
            AnimKind::EntityDeath { color, color_sec },
        )
    }

    fn new(owner: &Rc<RefCell<EntityState>>, duration_millis: ExtInt, kind: AnimKind) -> Anim {
        Anim {
            kind,
            elapsed: 0,
            duration_millis,
            marked_for_removal: Rc::new(Cell::new(false)),
            completion_callbacks: Vec::new(),
            update_callbacks: Vec::new(),
            owner: Rc::clone(owner),
            removal_effect: None,
        }
    }

    pub fn set_removal_effect(&mut self, index: usize) {
        self.removal_effect = Some(index);
    }

    pub fn update(&mut self, millis: u32) -> bool {
        if self.marked_for_removal.get() {
            return false;
        }

        let elapsed = self.elapsed + millis;
        self.elapsed = elapsed;

        self.update_kind(elapsed);

        if !self.update_callbacks.is_empty() && elapsed > self.update_callbacks[0].0 {
            self.update_callbacks[0].1.on_anim_update();
            self.update_callbacks.remove(0);
        }

        let remove = (self.duration_millis.less_than(elapsed) && self.ok_to_remove())
            || self.marked_for_removal.get();

        !remove
    }

    fn get_completion_callbacks(
        &mut self,
        update_cbs: &mut Vec<Box<dyn ScriptCallback>>,
        complete_cbs: &mut Vec<Box<dyn ScriptCallback>>,
    ) {
        for cb in self.update_callbacks.drain(..) {
            update_cbs.push(cb.1);
        }

        for cb in self.completion_callbacks.drain(..) {
            complete_cbs.push(cb);
        }
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
            EntityImageLayer { ref images } => {
                self.owner.borrow_mut().actor.add_anim_image_layers(images);
            }
            EntityColor {
                ref mut color,
                ref mut color_sec,
            } => entity_color_animation::update(color, color_sec, &self.owner, millis),
            EntityDeath {
                ref mut color,
                ref mut color_sec,
            } => entity_color_animation::update(color, color_sec, &self.owner, millis),
            EntitySubpos {
                ref mut x,
                ref mut y,
            } => entity_subpos_animation::update(x, y, &self.owner, millis),
            EntityScale { ref mut scale } => {
                entity_scale_animation::update(scale, &self.owner, millis)
            }
            MeleeAttack { ref mut model } => {
                melee_attack_animation::update(&self.owner, model, frac)
            }
            RangedAttack { ref mut model } => {
                ranged_attack_animation::update(&self.owner, model, frac)
            }
            Move { ref mut model } => {
                move_animation::update(&self.owner, &self.marked_for_removal, model, millis)
            }
            ParticleGenerator {
                ref mut model,
                ref mut state,
            } => particle_generator::update(
                &self.owner,
                model,
                state,
                &self.marked_for_removal,
                millis,
            ),
            _ => (),
        }
    }

    fn cleanup_kind(&mut self) {
        use self::AnimKind::*;
        match self.kind {
            EntityImageLayer { ref images } => {
                self.owner
                    .borrow_mut()
                    .actor
                    .remove_anim_image_layers(images);
            }
            EntityColor { .. } => entity_color_animation::cleanup(&self.owner),
            EntitySubpos { .. } => entity_subpos_animation::cleanup(&self.owner),
            EntityScale { .. } => entity_scale_animation::cleanup(&self.owner),
            MeleeAttack { .. } => melee_attack_animation::cleanup(&self.owner),
            RangedAttack { .. } => ranged_attack_animation::cleanup(&self.owner),
            Move { ref mut model } => move_animation::cleanup(&self.owner, model),
            EntityDeath { .. } => {
                entity_color_animation::cleanup(&self.owner);
                self.owner.borrow_mut().marked_for_removal = true;
            }
            _ => (),
        }
    }

    pub fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        use self::AnimKind::*;
        match self.kind {
            RangedAttack { ref model } => {
                ranged_attack_animation::draw(model, renderer, offset, scale, millis)
            }
            ParticleGenerator {
                ref state,
                ref model,
            } => {
                particle_generator::draw(state, model, &self.owner, renderer, offset, scale, millis)
            }
            Move { ref model } => move_animation::draw(model, renderer, offset, scale, millis),
            _ => (),
        }
    }

    pub fn add_completion_callback(&mut self, callback: Box<dyn ScriptCallback>) {
        self.completion_callbacks.push(callback);
    }

    pub fn add_update_callback(&mut self, callback: Box<dyn ScriptCallback>, time_millis: u32) {
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
        effect.removal_listeners.add(ChangeListener::new(
            "anim",
            Box::new(move |_| {
                marked_for_removal.set(true);
            }),
        ));
    }

    pub fn get_marked_for_removal(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.marked_for_removal)
    }

    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal.set(true);
    }

    pub fn owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }

    pub fn is_blocking(&self) -> bool {
        use self::AnimKind::*;
        match &self.kind {
            EntityColor { .. } => false,
            EntitySubpos { .. } => true,
            EntityScale { .. } | EntityImageLayer { .. } => !self.duration_millis.is_infinite(),
            MeleeAttack { .. } => true,
            RangedAttack { .. } => true,
            Move { .. } => true,
            Wait => true,
            NonBlockingWait => false,
            ParticleGenerator { model, .. } => model.is_blocking,
            EntityDeath { .. } => true,
        }
    }
}

/// Helper function to return the number of frames elapsed for the 'elapsed'
/// duration.  This is equal to the duration divided by 'frame_time'.  The
/// value is capped by 'max_index'
pub fn get_current_frame(elapsed: Duration, frame_time: u32, max_index: usize) -> usize {
    let millis = util::get_elapsed_millis(elapsed);

    let frame_index = (millis / frame_time) as usize;

    cmp::min(frame_index, max_index)
}
