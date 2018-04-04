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
use std::collections::{HashMap};

use rlua::{UserData, UserDataMethods};

use sulis_rules::HitKind;
use sulis_module::{Ability, Module};
use script::{Result, script_entity, ScriptEntity, ScriptEntitySet};
use {EntityState, GameState};

#[derive(Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq)]
enum FuncKind {
    BeforeAttack,
    AfterAttack,
    OnAnimComplete,
    OnAnimUpdate,
}

pub trait ScriptCallback {
    fn before_attack(&self) { }

    fn after_attack(&self, _hit_kind: HitKind) { }

    fn on_anim_complete(&mut self) { }

    fn on_anim_update(&self) { }
}

#[derive(Clone)]
pub struct CallbackData {
    parent: usize,
    ability_id: String,
    targets: ScriptEntitySet,
    funcs: HashMap<FuncKind, String>,
}

impl CallbackData {
    pub fn new(parent: usize, ability_id: &str) -> CallbackData {
        let targets = ScriptEntitySet {
            parent,
            point: None,
            indices: Vec::new(),
        };
        CallbackData {
            parent,
            ability_id: ability_id.to_string(),
            targets,
            funcs: HashMap::new(),
        }
    }

    fn get_params(&self) -> (Rc<RefCell<EntityState>>, Rc<Ability>, ScriptEntitySet) {
        let area_state = GameState::area_state();
        let parent = area_state.borrow().get_entity(self.parent);
        let ability = Module::ability(&self.ability_id).unwrap();
        (parent, ability, self.targets.clone())
    }

    fn add_func(&mut self, kind: FuncKind, name: String) -> Result<()> {
        self.funcs.insert(kind, name);
        Ok(())
    }
}

impl ScriptCallback for CallbackData {
    fn before_attack(&self) {
        let func = match self.funcs.get(&FuncKind::BeforeAttack) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn after_attack(&self, hit_kind: HitKind) {
        let func = match self.funcs.get(&FuncKind::AfterAttack) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_after_attack(&parent, &ability, targets, hit_kind, &func);
    }

    fn on_anim_complete(&mut self) {
        let func = match self.funcs.get(&FuncKind::OnAnimComplete) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn on_anim_update(&self) {
        let func = match self.funcs.get(&FuncKind::OnAnimUpdate) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability, targets) = self.get_params();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }
}

impl UserData for CallbackData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("add_target", |_, cb, target: ScriptEntity| {
            let index = target.try_unwrap_index()?;
            cb.targets.indices.push(Some(index));
            Ok(())
        });

        methods.add_method_mut("add_targets", |_, cb, targets: ScriptEntitySet| {
            cb.targets.indices.append(&mut targets.indices.clone());
            cb.targets.point = targets.point.clone();
            Ok(())
        });

        methods.add_method_mut("add_selected_point", |_, cb, p: HashMap<String, i32>| {
            let (x, y) = script_entity::unwrap_point(p)?;
            cb.targets.point = Some((x, y));
            Ok(())
        });

        methods.add_method_mut("set_before_attack_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::BeforeAttack, func));
        methods.add_method_mut("set_after_attack_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::AfterAttack, func));
        methods.add_method_mut("set_on_anim_update_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnAnimUpdate, func));
        methods.add_method_mut("set_on_anim_complete_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnAnimComplete, func));
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
