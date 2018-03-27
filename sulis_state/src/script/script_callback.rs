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
use std::collections::HashSet;

use rlua::{UserData, UserDataMethods};

use sulis_rules::HitKind;
use sulis_module::{Ability, Module};
use script::{ScriptEntity, ScriptEntitySet};
use {EntityState, GameState};

const BEFORE_ATTACK: &str = "before_attack";
const AFTER_ATTACK: &str = "after_attack";
const ON_ANIM_COMPLETE: &str = "on_anim_complete";
const ON_ANIM_UPDATE: &str = "on_anim_update";

pub trait ScriptCallback {
    fn before_attack(&self) { }

    fn after_attack(&self, _hit_kind: HitKind) { }

    fn on_anim_complete(&self) { }

    /// Callback called by an animation at a predefined time.
    /// The `index` is the index of the callback within the animation's
    /// list of callbacks.  It is 1 based to match with Lua indexing
    fn on_anim_update(&self, _index: usize) { }
}

#[derive(Clone)]
pub struct CallbackData {
    parent: usize,
    ability_id: String,
    targets: Vec<Option<usize>>,
    funcs: HashSet<String>,
}

impl CallbackData {
    pub fn new(parent: usize, ability_id: &str) -> CallbackData {
        CallbackData {
            parent,
            ability_id: ability_id.to_string(),
            targets: Vec::new(),
            funcs: HashSet::new(),
        }
    }

    fn get_params(&self) -> (Rc<RefCell<EntityState>>, Rc<Ability>, Vec<Option<Rc<RefCell<EntityState>>>>) {
        let area_state = GameState::area_state();
        let parent = area_state.borrow().get_entity(self.parent);
        let ability = Module::ability(&self.ability_id).unwrap();
        let mut targets = Vec::new();
        for index in self.targets.iter() {
            let index = match index {
                &None => {
                    targets.push(None);
                    continue;
                }, &Some(index) => index,
            };

            match area_state.borrow().check_get_entity(index) {
                None => targets.push(None),
                Some(entity) => targets.push(Some(entity)),
            }
        }
        (parent, ability, targets)
    }
}

impl ScriptCallback for CallbackData {
    fn before_attack(&self) {
        if !self.funcs.contains(BEFORE_ATTACK) { return; }

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_script(&parent, &ability, targets, "before_attack");
    }

    fn after_attack(&self, hit_kind: HitKind) {
        if !self.funcs.contains(AFTER_ATTACK) { return; }

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_after_attack(&parent, &ability, targets, hit_kind);
    }

    fn on_anim_complete(&self) {
        if !self.funcs.contains(ON_ANIM_COMPLETE) { return; }

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_script(&parent, &ability, targets, "on_anim_complete");
    }

    fn on_anim_update(&self, index: usize) {
        if !self.funcs.contains(ON_ANIM_UPDATE) { return; }

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_on_anim_update(&parent, &ability, targets, index);
    }
}

impl UserData for CallbackData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("add_target", |_, cb, target: ScriptEntity| {
            let index = target.try_unwrap_index()?;
            cb.targets.push(Some(index));
            Ok(())
        });

        methods.add_method_mut("add_targets", |_, cb, targets: ScriptEntitySet| {
            cb.targets.append(&mut targets.indices.clone());
            Ok(())
        });

        methods.add_method_mut("register_fn", |_, cb, func: String| {
            cb.funcs.insert(func);
            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct ScriptHitKind {
    pub kind: HitKind,
}

impl UserData for ScriptHitKind {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("is_miss", |_, hit, ()| Ok(hit.kind == HitKind::Miss));
        methods.add_method("is_graze", |_, hit, ()| Ok(hit.kind == HitKind::Graze));
        methods.add_method("is_hit", |_, hit, ()| Ok(hit.kind == HitKind::Hit));
        methods.add_method("is_crit", |_, hit, ()| Ok(hit.kind == HitKind::Crit));
    }
}
