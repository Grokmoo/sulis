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
use std::io::Error;
use std::result::{self};

use rlua::{UserData, UserDataMethods};

use sulis_core::util::invalid_data_error;
use sulis_rules::HitKind;
use sulis_module::{Module};
use script::{Result, script_entity, ScriptEntity, ScriptEntitySet, ScriptActiveSurface, ScriptItemKind};
use {EntityState, GameState};

pub fn fire_round_elapsed(cbs: Vec<Rc<CallbackData>>) {
    for cb in cbs {
        cb.on_round_elapsed();
        cb.on_surface_round_elapsed();
    }
}

pub fn fire_on_moved_in_surface(cbs: Vec<(Rc<CallbackData>, usize)>) {
    for (cb, target) in cbs {
        cb.on_moved_in_surface(target);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub enum FuncKind {
    OnDamaged,
    BeforeAttack,
    AfterAttack,
    AfterDefense,
    BeforeDefense,
    OnAnimComplete,
    OnAnimUpdate,
    OnRoundElapsed,
    OnSurfaceRoundElapsed,
    OnMovedInSurface,
}

pub trait ScriptCallback {
    fn on_damaged(&self, _targets: &ScriptEntitySet, _hit_kind: HitKind, _damage: u32) { }

    fn after_defense(&self, _targets: &ScriptEntitySet, _hit_kind: HitKind, _damage: u32) { }

    fn before_defense(&self, _targets: &ScriptEntitySet) { }

    fn before_attack(&self, _targets: &ScriptEntitySet) { }

    fn after_attack(&self, _targets: &ScriptEntitySet, _hit_kind: HitKind, _damage: u32) { }

    fn on_anim_complete(&self) { }

    fn on_anim_update(&self) { }

    fn on_round_elapsed(&self) { }

    fn on_surface_round_elapsed(&self) { }

    fn on_moved_in_surface(&self, _target: usize) { }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
enum Kind {
    Ability(String),
    Item(String), // callback is based on an item ID, not a particular
                  // slot - this allows creating callbacks after the
                  // consumable items has been used
    Entity,
    Script(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CallbackData {
    parent: usize,
    effect: Option<usize>,
    kind: Kind,
    targets: Option<ScriptEntitySet>,
    funcs: HashMap<FuncKind, String>,
}

impl CallbackData {
    pub fn update_entity_refs_on_load(&mut self, entities: &HashMap<usize,
                                      Rc<RefCell<EntityState>>>) -> result::Result<(), Error> {

        match entities.get(&self.parent) {
            None => {
                return invalid_data_error(&format!("Invalid parent {} for callback", self.parent));
            }, Some(ref entity) => self.parent = entity.borrow().index(),
        }

        if let Some(ref mut targets) = &mut self.targets {
            targets.update_entity_refs_on_load(entities)?;
        }
        Ok(())
    }

    pub fn update_effect_index_on_load(&mut self, index: usize) {
        self.effect = Some(index);
    }

    pub fn new_ability(parent: usize, ability_id: &str) -> CallbackData {
        CallbackData {
            parent,
            effect: None,
            kind: Kind::Ability(ability_id.to_string()),
            targets: None,
            funcs: HashMap::new(),
        }
    }

    pub fn new_item(parent: usize, item_id: String) -> CallbackData {
        CallbackData {
            parent,
            effect: None,
            kind: Kind::Item(item_id),
            targets: None,
            funcs: HashMap::new(),
        }
    }

    pub fn new_entity(parent: usize) -> CallbackData {
        CallbackData {
            parent,
            effect: None,
            kind: Kind::Entity,
            targets: None,
            funcs: HashMap::new(),
        }
    }

    pub fn new_trigger(parent: usize, script: String) -> CallbackData {
        CallbackData {
            parent,
            effect: None,
            kind: Kind::Script(script),
            targets: None,
            funcs: HashMap::new(),
        }
    }

    // functions used in setting up the data

    pub(crate) fn set_effect(&mut self, index: usize) {
        self.effect = Some(index);
    }

    pub fn add_func(&mut self, kind: FuncKind, name: String) -> Result<()> {
        self.funcs.insert(kind, name);
        Ok(())
    }

    fn create_targets_if_missing(&mut self) {
        if self.targets.is_some() { return; }

        self.targets = Some(ScriptEntitySet::with_parent(self.parent));
    }

    // functions used in firing the script callback

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

    fn exec_standard_script(&self, targets: ScriptEntitySet, func_kind: FuncKind) {
        let func = match self.funcs.get(&func_kind) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);

        match &self.kind {
            Kind::Ability(ref id) => {
                let ability = Module::ability(id).unwrap();
                GameState::execute_ability_script(&parent, &ability, targets, &func);
            },
            Kind::Item(id) => {
                GameState::execute_item_script(&parent, ScriptItemKind::WithID(id.to_string()),
                    targets, &func);
            },
            Kind::Entity => {
                GameState::execute_entity_script(&parent, targets, &func);
            },
            Kind::Script(script) => {
                GameState::execute_trigger_script(&script, &func, &parent, &parent);
            }
        }
    }

    fn exec_script_with_attack_data(&self, targets: ScriptEntitySet, hit_kind: HitKind,
                                    damage: u32, func_kind: FuncKind) {
        let func = match self.funcs.get(&func_kind) {
            None => return,
            Some(ref func) => func.to_string(),
        };

        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);

        match &self.kind {
            Kind::Ability(ref id) => {
                let ability = Module::ability(id).unwrap();
                GameState::execute_ability_with_attack_data(&parent, &ability, targets,
                                                            hit_kind, damage, &func);
            },
            Kind::Item(id) => {
                GameState::execute_item_with_attack_data(&parent,
                    ScriptItemKind::WithID(id.to_string()), targets, hit_kind, damage, &func);
            },
            Kind::Entity => {
                GameState::execute_entity_with_attack_data(&parent, targets, hit_kind, damage,
                                                           &func);
            },
            Kind::Script(script) => {
                GameState::execute_trigger_script(&script, &func, &parent, &parent);
            }
        }
    }
}

impl ScriptCallback for CallbackData {
    fn before_defense(&self, targets: &ScriptEntitySet) {
        self.exec_standard_script(self.get_targets(targets), FuncKind::BeforeDefense);
    }

    fn before_attack(&self, targets: &ScriptEntitySet) {
        self.exec_standard_script(self.get_targets(targets), FuncKind::BeforeAttack);
    }

    fn on_anim_complete(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnAnimComplete);
    }

    fn on_anim_update(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnAnimUpdate);
    }

    fn on_round_elapsed(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnRoundElapsed);
    }

    /// when called, this computes the current target set and sends it to
    /// the lua function based on the surface state
    fn on_surface_round_elapsed(&self) {
        if self.funcs.get(&FuncKind::OnSurfaceRoundElapsed).is_none() { return; }

        let targets = match compute_surface_targets(self.effect, self.parent, None) {
            Some(targets) => targets,
            None => {
                warn!("Unable to fire on surface_round_elapsed");
                return;
            }
        };
        self.exec_standard_script(targets, FuncKind::OnSurfaceRoundElapsed);
    }

    fn on_moved_in_surface(&self, target: usize) {
        if self.funcs.get(&FuncKind::OnMovedInSurface).is_none() { return; }

        let targets = match compute_surface_targets(self.effect, self.parent, Some(target)) {
            Some(targets) => targets,
            None => {
                warn!("Unable to fire on_moved_in_surface");
                return;
            }
        };

        self.exec_standard_script(targets, FuncKind::OnMovedInSurface);
    }

    fn after_defense(&self, targets: &ScriptEntitySet, hit_kind: HitKind, damage: u32) {
        self.exec_script_with_attack_data(self.get_targets(targets), hit_kind, damage,
            FuncKind::AfterDefense);
    }

    fn after_attack(&self, targets: &ScriptEntitySet, hit_kind: HitKind, damage: u32) {
        self.exec_script_with_attack_data(self.get_targets(targets), hit_kind, damage,
            FuncKind::AfterAttack);
    }

    fn on_damaged(&self, targets: &ScriptEntitySet, hit_kind: HitKind, damage: u32) {
        self.exec_script_with_attack_data(self.get_targets(targets), hit_kind, damage,
            FuncKind::OnDamaged);
    }
}

fn compute_surface_targets(effect: Option<usize>, parent: usize, target: Option<usize>) -> Option<ScriptEntitySet> {
    let effect = match effect {
        None => {
            warn!("Surface effect is not set");
            return None;
        },
        Some(index) => index,
    };

    let mut targets = ScriptEntitySet::with_parent(parent);
    targets.surface = Some(ScriptActiveSurface { index: effect });
    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();

    let effect = match mgr.effect_checked(effect) {
        None => {
            warn!("Invalid effect for surface");
            return None;
        },
        Some(effect) => effect,
    };

    match effect.surface() {
        None => {
            warn!("Attempted to exec on_surface_round_elapsed on non-surface");
            return None;
        }, Some((area_id, points)) => {
            targets.affected_points = points.iter().map(|p| (p.x, p.y)).collect();

            let area = GameState::get_area_state(area_id).unwrap();
            if let Some(target) = target {
                targets.indices.push(Some(target));
            } else {
                let inside = area.borrow().entities_with_points(points);
                targets.indices = inside.into_iter().map(|i| Some(i)).collect();
            }
        }
    }

    Some(targets)
}

impl UserData for CallbackData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("add_target", |_, cb, target: ScriptEntity| {
            if let Kind::Script(_) = cb.kind {
                warn!("Setting targets on global generated callback will have no effect");
            }
            cb.create_targets_if_missing();
            let index = target.try_unwrap_index()?;
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.indices.push(Some(index));
            }
            Ok(())
        });

        methods.add_method_mut("add_targets", |_, cb, targets: ScriptEntitySet| {
            if let Kind::Script(_) = cb.kind {
                warn!("Setting targets on global generated callback will have no effect");
            }

            cb.create_targets_if_missing();
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.append(&targets);
            }
            Ok(())
        });

        methods.add_method_mut("add_selected_point", |_, cb, p: HashMap<String, i32>| {
            if let Kind::Script(_) = cb.kind {
                warn!("Setting targets on global generated callback will have no effect");
            }

            cb.create_targets_if_missing();
            let (x, y) = script_entity::unwrap_point(p)?;
            if let Some(ref mut cb_targets) = cb.targets {
                cb_targets.selected_point = Some((x, y));
            }
            Ok(())
        });

        methods.add_method_mut("set_on_damaged_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnDamaged, func));
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
        methods.add_method_mut("set_on_moved_in_surface_fn",
                               |_, cb, func: String| cb.add_func(FuncKind::OnMovedInSurface, func));
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
        methods.add_method("kind", |_, hit, ()| {
            Ok(format!("{:?}", hit.kind))
        });
    }
}
