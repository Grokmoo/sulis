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

pub fn fire_round_elapsed(cbs: Vec<Rc<ScriptCallback>>) {
    for cb in cbs {
        cb.on_round_elapsed();
        cb.on_surface_round_elapsed();
    }
}

#[derive(Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq)]
enum FuncKind {
    BeforeAttack,
    AfterAttack,
    AfterDefense,
    BeforeDefense,
    OnAnimComplete,
    OnAnimUpdate,
    OnRoundElapsed,
    OnSurfaceRoundElapsed,
}

pub trait ScriptCallback {
    fn after_defense(&self, _targets: &ScriptEntitySet, _hit_kind: HitKind, _damage: u32) { }

    fn before_defense(&self, _targets: &ScriptEntitySet) { }

    fn before_attack(&self, _targets: &ScriptEntitySet) { }

    fn after_attack(&self, _targets: &ScriptEntitySet, _hit_kind: HitKind, _damage: u32) { }

    fn on_anim_complete(&self) { }

    fn on_anim_update(&self) { }

    fn on_round_elapsed(&self) { }

    fn on_surface_round_elapsed(&self) { }
}

#[derive(Clone)]
pub struct CallbackData {
    parent: usize,
    effect: Option<usize>,
    ability_id: String,
    targets: Option<ScriptEntitySet>,
    funcs: HashMap<FuncKind, String>,
}

impl CallbackData {
    pub fn new(parent: usize, ability_id: &str) -> CallbackData {
        CallbackData {
            parent,
            effect: None,
            ability_id: ability_id.to_string(),
            targets: None,
            funcs: HashMap::new(),
        }
    }

    pub(crate) fn set_effect(&mut self, index: usize) {
        self.effect = Some(index);
    }

    fn get_params(&self) -> (Rc<RefCell<EntityState>>, Rc<Ability>) {
        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);
        let ability = Module::ability(&self.ability_id).unwrap();
        (parent, ability)
    }

    fn add_func(&mut self, kind: FuncKind, name: String) -> Result<()> {
        self.funcs.insert(kind, name);
        Ok(())
    }

    fn create_targets_if_missing(&mut self) {
        if self.targets.is_some() { return; }

        self.targets = Some(ScriptEntitySet::with_parent(self.parent));
    }

    fn get_or_create_targets(&self) -> ScriptEntitySet {
        if let Some(ref targets) = self.targets {
            targets.clone()
        } else {
            ScriptEntitySet::with_parent(self.parent)
        }
    }

    fn get_targets(&self, targets: &ScriptEntitySet) -> ScriptEntitySet {
        if let Some(ref targets) = self.targets {
            targets.clone()
        } else {
            targets.clone()
        }
    }
}

impl ScriptCallback for CallbackData {
    fn after_defense(&self, targets: &ScriptEntitySet, hit_kind: HitKind, damage: u32) {
        let func = match self.funcs.get(&FuncKind::AfterDefense) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_targets(targets);
        GameState::execute_ability_with_attack_data(&parent, &ability, targets,
                                                    hit_kind, damage, &func);
    }

    fn before_defense(&self, targets: &ScriptEntitySet) {
        let func = match self.funcs.get(&FuncKind::BeforeDefense) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_targets(targets);
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn before_attack(&self, targets: &ScriptEntitySet) {
        let func = match self.funcs.get(&FuncKind::BeforeAttack) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_targets(targets);
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn after_attack(&self, targets: &ScriptEntitySet, hit_kind: HitKind, damage: u32) {
        let func = match self.funcs.get(&FuncKind::AfterAttack) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_targets(targets);
        GameState::execute_ability_with_attack_data(&parent, &ability, targets,
                                                    hit_kind, damage, &func);
    }

    fn on_anim_complete(&self) {
        let func = match self.funcs.get(&FuncKind::OnAnimComplete) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_or_create_targets();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn on_anim_update(&self) {
        let func = match self.funcs.get(&FuncKind::OnAnimUpdate) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_or_create_targets();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    fn on_round_elapsed(&self) {
        let func = match self.funcs.get(&FuncKind::OnRoundElapsed) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        let targets = self.get_or_create_targets();
        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }

    /// when called, this computes the current target set and sends it to
    /// the lua function based on the surface state
    fn on_surface_round_elapsed(&self) {
        let func = match self.funcs.get(&FuncKind::OnSurfaceRoundElapsed) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let (parent, ability) = self.get_params();
        match compute_surface_targets(self.effect, self.parent) {
            Some(targets) => GameState::execute_ability_script(&parent, &ability, targets, &func),
            None => warn!("Unable to fire on_surface_round_elapsed"),
        };
    }
}

fn compute_surface_targets(effect: Option<usize>, parent: usize) -> Option<ScriptEntitySet> {
    let effect = match effect {
        None => return None,
        Some(index) => index,
    };

    let mut targets = ScriptEntitySet::with_parent(parent);

    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();

    let effect = match mgr.effect_checked(effect) {
        None => return None,
        Some(effect) => effect,
    };

    match effect.surface() {
        None => {
            warn!("Attempted to exec on_surface_round_elapsed on non-surface");
            return None;
        }, Some((area_id, points)) => {
            targets.affected_points = points.iter().map(|p| (p.x, p.y)).collect();

            let area = GameState::get_area_state(area_id).unwrap();
            let inside = area.borrow().entities_with_points(points);
            targets.indices = inside.into_iter().map(|i| Some(i)).collect();
        }
    }

    Some(targets)
}

impl UserData for CallbackData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("add_target", |_, cb, target: ScriptEntity| {
            cb.create_targets_if_missing();
            let index = target.try_unwrap_index()?;
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.indices.push(Some(index));
            }
            Ok(())
        });

        methods.add_method_mut("add_targets", |_, cb, targets: ScriptEntitySet| {
            cb.create_targets_if_missing();
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.append(&targets);
            }
            Ok(())
        });

        methods.add_method_mut("add_selected_point", |_, cb, p: HashMap<String, i32>| {
            cb.create_targets_if_missing();
            let (x, y) = script_entity::unwrap_point(p)?;
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.selected_point = Some((x, y));
            }
            Ok(())
        });

        methods.add_method_mut("set_before_attack_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::BeforeAttack, func));
        methods.add_method_mut("set_after_attack_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::AfterAttack, func));
        methods.add_method_mut("set_before_defense_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::BeforeDefense, func));
        methods.add_method_mut("set_after_defense_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::AfterDefense, func));
        methods.add_method_mut("set_on_anim_update_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnAnimUpdate, func));
        methods.add_method_mut("set_on_anim_complete_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnAnimComplete, func));
        methods.add_method_mut("set_on_round_elapsed_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnRoundElapsed, func));
        methods.add_method_mut("set_on_surface_round_elapsed_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnSurfaceRoundElapsed, func));
    }
}

#[derive(Clone)]
pub struct ScriptHitKind {
    pub kind: HitKind,
    pub damage: u32,
}

impl UserData for ScriptHitKind {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("is_miss", |_, hit, ()| Ok(hit.kind == HitKind::Miss));
        methods.add_method("is_graze", |_, hit, ()| Ok(hit.kind == HitKind::Graze));
        methods.add_method("is_hit", |_, hit, ()| Ok(hit.kind == HitKind::Hit));
        methods.add_method("is_crit", |_, hit, ()| Ok(hit.kind == HitKind::Crit));
        methods.add_method("total_damage", |_, hit, ()| Ok(hit.damage));
    }
}
