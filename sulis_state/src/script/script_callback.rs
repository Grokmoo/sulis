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
use std::rc::Rc;
use std::result;

use serde::{Serialize, Deserialize};
use rlua::{UserData, UserDataMethods};

use crate::script::{
    script_entity, ScriptActiveSurface, ScriptAppliedEffect, ScriptEntity, ScriptEntitySet,
    ScriptItemKind, ScriptMenuSelection,
};
use crate::{EntityState, GameState, Script};
use sulis_core::util::invalid_data_error;
use sulis_module::{on_trigger::Kind, Ability, DamageKind, HitKind, Module};

pub fn fire_round_elapsed(cbs: Vec<Rc<CallbackData>>) {
    for cb in cbs {
        cb.on_round_elapsed();
        cb.on_surface_round_elapsed();
    }
}

pub struct TriggeredCallback {
    cb: Rc<CallbackData>,
    target: usize,
    func: FuncKind,
}

impl TriggeredCallback {
    pub fn new(cb: Rc<CallbackData>, func: FuncKind) -> TriggeredCallback {
        TriggeredCallback {
            cb,
            target: 0,
            func,
        }
    }

    pub fn with_target(cb: Rc<CallbackData>, func: FuncKind, target: usize) -> TriggeredCallback {
        TriggeredCallback { cb, target, func }
    }
}

pub fn fire_cbs(cbs: Vec<TriggeredCallback>) {
    for cb in cbs {
        let target = cb.target;
        let func = cb.func;
        let cb = cb.cb;
        match func {
            FuncKind::OnMovedInSurface => cb.on_moved_in_surface(target),
            FuncKind::OnEnteredSurface => cb.on_entered_surface(target),
            FuncKind::OnExitedSurface => cb.on_exited_surface(target),
            FuncKind::OnRemoved => cb.on_removed(),
            FuncKind::OnMoved => cb.on_moved(),
            FuncKind::OnRoundElapsed => cb.on_round_elapsed(),
            FuncKind::OnSurfaceRoundElapsed => cb.on_surface_round_elapsed(),
            FuncKind::OnActivated => match &cb.kind {
                Kind::Ability(id) => {
                    let ability = Module::ability(id).unwrap();
                    let func = get_on_activate_fn(&cb, &ability);
                    Script::ability_on_activate(cb.parent, func, &ability);
                }
                _ => warn!("OnActivated called with invalid callback kind."),
            },
            FuncKind::OnDeactivated => match &cb.kind {
                Kind::Ability(id) => {
                    let ability = Module::ability(id).unwrap();
                    Script::ability_on_deactivate(cb.parent, &ability);
                }
                _ => warn!("OnDeactivated called with invalid callback kind."),
            },
            _ => {
                warn!("Surface callback of kind {:?} is not being called.", func);
            }
        }
    }
}

const ON_ACTIVATE_DEFAULT: &str = "on_activate";

fn get_on_activate_fn(cb: &CallbackData, ability: &Ability) -> String {
    let ai_data = match &ability.active {
        None => return ON_ACTIVATE_DEFAULT.to_string(),
        Some(active) => &active.ai,
    };

    let func = match &ai_data.on_activate_fn {
        None => return ON_ACTIVATE_DEFAULT.to_string(),
        Some(func) => func.to_string(),
    };

    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();
    match mgr.entity_checked(cb.parent) {
        None => ON_ACTIVATE_DEFAULT.to_string(),
        Some(entity) => {
            if entity.borrow().is_party_member() {
                ON_ACTIVATE_DEFAULT.to_string()
            } else {
                func
            }
        }
    }
}

pub fn fire_on_moved(cbs: Vec<Rc<CallbackData>>) {
    for cb in cbs {
        cb.on_moved();
    }
}

/// A type of callback function for a `CallbackData` object.

#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub enum FuncKind {
    /// Save compatibility only.  TODO remove this after a while
    OnSwapWeapons,

    /// Called when an entity swaps their weapon set
    OnHeldChanged,

    /// Called when a new effect is applied to a parent
    OnEffectApplied,

    /// Called when a menu option is selected in a custom menu
    OnMenuSelect,

    /// Called whenever an effect is removed from a parent
    OnRemoved,

    /// Called whenver a parent entity loses hit points
    OnDamaged,

    /// Called on the attacking entity immediately before the attack is rolled
    /// Only applies to standard weapon attacks.
    BeforeAttack,

    /// Called on the attacking entity immediately after the attack is rolled.
    /// Only applies to standard weapon attacks.
    AfterAttack,

    /// Called on the defending entity immediately after the attack is rolled.
    /// Only applies to standard weapon attacks.
    AfterDefense,

    /// Called on the defending entity immediately before the attack is rolled.
    /// Only applies to standard weapon attacks.
    BeforeDefense,

    /// Called whenever an animation copmletes, due to its duration elapsing or
    /// the owning effect being removed.
    OnAnimComplete,

    /// Called at a specific time elapsed on an animation update.
    OnAnimUpdate,

    /// Called once each time a round elapses for the parent
    OnRoundElapsed,

    /// Called each time an entity moves a square
    OnMoved,

    /// Only relevant for surfaces.  Called once each time a round elapses for the
    /// surface effect, including a list of all affected entities as targets.
    OnSurfaceRoundElapsed,

    /// Called each time an entity moves within a given surface.  Controlled by
    /// `set_squares_to_fire_on_moved`
    OnMovedInSurface,

    /// Called when an entity enters a surface
    OnEnteredSurface,

    /// Called when an entity exits a surface
    OnExitedSurface,

    /// Called when an ability is activated
    OnActivated,

    /// Called whena an ability mode is deactivated
    OnDeactivated,
}

/// A trait representing a callback that will fire a script when called.  In lua scripts,
/// `CallbackData` is constructed to use this trait.
pub trait ScriptCallback {
    fn on_held_changed(&self) {}

    fn on_effect_applied(&self, _effect: ScriptAppliedEffect) {}

    fn on_menu_select(&self, _value: ScriptMenuSelection) {}

    fn on_removed(&self) {}

    fn on_damaged(
        &self,
        _targets: &ScriptEntitySet,
        _hit_kind: HitKind,
        _damage: Vec<(DamageKind, u32)>,
    ) {
    }

    fn after_defense(
        &self,
        _targets: &ScriptEntitySet,
        _hit_kind: HitKind,
        _damage: Vec<(DamageKind, u32)>,
    ) {
    }

    fn before_defense(&self, _targets: &ScriptEntitySet) {}

    fn before_attack(&self, _targets: &ScriptEntitySet) {}

    fn after_attack(
        &self,
        _targets: &ScriptEntitySet,
        _hit_kind: HitKind,
        _damage: Vec<(DamageKind, u32)>,
    ) {
    }

    fn on_anim_complete(&self) {}

    fn on_anim_update(&self) {}

    fn on_round_elapsed(&self) {}

    fn on_moved(&self) {}

    fn on_surface_round_elapsed(&self) {}

    fn on_moved_in_surface(&self, _target: usize) {}

    fn on_entered_surface(&self, _target: usize) {}

    fn on_exited_surface(&self, _target: usize) {}
}

/// A callback that can be passed to various functions to be executed later.
/// A single callback can hold multiple invocations by setting several different functions.
///
/// # `add_target(target: ScriptEntity)`
/// Adds the specified `target` to the list of targets this callback will provide to its
/// callee.
///
/// # `add_targets(targets: ScriptEntitySet)`
/// Adds the specified `targets` to the list of targets this callback will provide to its
/// callee
///
/// # `add_selected_point(point: Table)`
/// Adds the specified `point` to the list of points this callback will provide in its
/// targets.  The point is a table of the form `{x:x_coord, y: y_coord}`
///
/// # `add_affected_points(points: Table)`
/// Adds the list of affected points to the affected_points this callback will provide its
/// targets.  The points is a list of tables of the form `{x: x_coord, y: y_coord}`
///
/// # `set_on_held_changed(func: String)`
/// # `set_on_effect_applied_fn(func: String)`
/// # `set_on_menu_select_fn(func: String)`
/// # `set_on_removed_fn(func: String)`
/// # `set_on_damaged_fn(func: String)`
/// # `set_before_attack_fn(func: String)`
/// # `set_after_attack_fn(func: String)`
/// # `set_before_defense_fn(func: String)`
/// # `set_after_defense_fn(func: String)`
/// # `set_on_anim_update_fn(func: String)`
/// # `set_on_anim_complete_fn(func: String)`
/// # `set_on_round_elapsed_fn(func: String)`
/// # `set_on_moved_fn(func: String)`
/// # `set_on_surface_round_elapsed_fn(func: String)`
/// # `set_on_moved_in_surface_fn(func: String)`
/// # `set_on_entered_surface_fn(func: String)`
/// # `set_on_exited_surface_fn(func: String)`
/// Each of these methods causes a specified lua `func` to be called when the condition is met,
/// as described in `FuncKind`.  Multiple of these methods may be added to one
/// Callback.
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
    pub fn clear_funcs_except(&mut self, keep: &[FuncKind]) {
        self.funcs.retain(|func, _| keep.contains(func));
    }

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    pub fn parent(&self) -> usize {
        self.parent
    }

    pub fn get_func(&self, func: FuncKind) -> Option<String> {
        self.funcs.get(&func).cloned()
    }

    pub fn update_entity_refs_on_load(
        &mut self,
        entities: &HashMap<usize, Rc<RefCell<EntityState>>>,
    ) -> result::Result<(), Error> {
        match entities.get(&self.parent) {
            None => {
                return invalid_data_error(&format!("Invalid parent {} for callback", self.parent));
            }
            Some(entity) => self.parent = entity.borrow().index(),
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

    pub fn add_func(&mut self, kind: FuncKind, name: String) {
        self.funcs.insert(kind, name);
    }

    fn create_targets_if_missing(&mut self) {
        if self.targets.is_some() {
            return;
        }

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
            Some(func) => func.to_string(),
        };

        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);

        match &self.kind {
            Kind::Ability(ref id) => {
                let ability = Module::ability(id).unwrap();
                Script::ability(&parent, &ability, targets, &func);
            }
            Kind::Item(id) => {
                Script::item(
                    &parent,
                    ScriptItemKind::WithID(id.to_string()),
                    targets,
                    &func,
                );
            }
            Kind::Entity => {
                Script::entity(&parent, targets, &func);
            }
            Kind::Script(script) => {
                Script::trigger(script, &func, ScriptEntity::from(&parent));
            }
        }
    }

    fn exec_script_with_arg<T>(&self, targets: ScriptEntitySet, arg: T, func_kind: FuncKind)
    where
        T: rlua::UserData + Send + 'static,
    {
        let func = match self.funcs.get(&func_kind) {
            None => return,
            Some(func) => func.to_string(),
        };

        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);

        match &self.kind {
            Kind::Ability(ref id) => {
                let ability = Module::ability(id).unwrap();
                Script::ability_with_arg(&parent, &ability, targets, arg, &func);
            }
            Kind::Item(id) => {
                Script::item_with_arg(
                    &parent,
                    ScriptItemKind::WithID(id.to_string()),
                    targets,
                    arg,
                    &func,
                );
            }
            Kind::Entity => {
                Script::entity_with_arg(&parent, targets, arg, &func);
            }
            Kind::Script(script) => {
                Script::trigger(script, &func, (ScriptEntity::from(&parent), arg));
            }
        }
    }

    fn exec_script_with_attack_data(
        &self,
        targets: ScriptEntitySet,
        hit_kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
        func_kind: FuncKind,
    ) {
        let func = match self.funcs.get(&func_kind) {
            None => return,
            Some(func) => func.to_string(),
        };

        let mgr = GameState::turn_manager();
        let parent = mgr.borrow().entity(self.parent);

        match &self.kind {
            Kind::Ability(ref id) => {
                let ability = Module::ability(id).unwrap();
                Script::ability_with_attack_data(
                    &parent, &ability, targets, hit_kind, damage, &func,
                );
            }
            Kind::Item(id) => {
                Script::item_with_attack_data(
                    &parent,
                    ScriptItemKind::WithID(id.to_string()),
                    targets,
                    hit_kind,
                    damage,
                    &func,
                );
            }
            Kind::Entity => {
                Script::entity_with_attack_data(&parent, targets, hit_kind, damage, &func);
            }
            Kind::Script(script) => {
                Script::trigger(
                    script,
                    &func,
                    (
                        ScriptEntity::from(&parent),
                        ScriptHitKind::new(hit_kind, damage),
                    ),
                );
            }
        }
    }

    fn exec_surface_script(&self, kind: FuncKind, target: Option<usize>) {
        if !self.funcs.contains_key(&kind) {
            return;
        }

        let targets = match compute_surface_targets(self.effect, self.parent, target) {
            Some(targets) => targets,
            None => {
                warn!("Unable to fire {:?}", kind);
                return;
            }
        };

        self.exec_standard_script(targets, kind);
    }
}

impl ScriptCallback for CallbackData {
    fn on_held_changed(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnHeldChanged);
    }

    fn on_effect_applied(&self, effect: ScriptAppliedEffect) {
        self.exec_script_with_arg(
            self.get_or_create_targets(),
            effect,
            FuncKind::OnEffectApplied,
        );
    }

    fn on_menu_select(&self, value: ScriptMenuSelection) {
        self.exec_script_with_arg(self.get_or_create_targets(), value, FuncKind::OnMenuSelect);
    }

    fn on_removed(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnRemoved);
    }

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

    fn on_moved(&self) {
        self.exec_standard_script(self.get_or_create_targets(), FuncKind::OnMoved);
    }

    /// when called, this computes the current target set and sends it to
    /// the lua function based on the surface state
    fn on_surface_round_elapsed(&self) {
        self.exec_surface_script(FuncKind::OnSurfaceRoundElapsed, None);
    }

    fn on_moved_in_surface(&self, target: usize) {
        self.exec_surface_script(FuncKind::OnMovedInSurface, Some(target));
    }

    fn on_entered_surface(&self, target: usize) {
        self.exec_surface_script(FuncKind::OnEnteredSurface, Some(target));
    }

    fn on_exited_surface(&self, target: usize) {
        // since it is called after the surface has been removed in some cases
        // we cannot preserve the surface info for on_exited_surface scripts
        if !self.funcs.contains_key(&FuncKind::OnExitedSurface) {
            return;
        }

        let mut targets = ScriptEntitySet::with_parent(self.parent);
        targets.indices.push(Some(target));

        self.exec_standard_script(targets, FuncKind::OnExitedSurface);
    }

    fn after_defense(
        &self,
        targets: &ScriptEntitySet,
        hit_kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
    ) {
        self.exec_script_with_attack_data(
            self.get_targets(targets),
            hit_kind,
            damage,
            FuncKind::AfterDefense,
        );
    }

    fn after_attack(
        &self,
        targets: &ScriptEntitySet,
        hit_kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
    ) {
        self.exec_script_with_attack_data(
            self.get_targets(targets),
            hit_kind,
            damage,
            FuncKind::AfterAttack,
        );
    }

    fn on_damaged(
        &self,
        targets: &ScriptEntitySet,
        hit_kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
    ) {
        self.exec_script_with_attack_data(
            self.get_targets(targets),
            hit_kind,
            damage,
            FuncKind::OnDamaged,
        );
    }
}

fn compute_surface_targets(
    effect: Option<usize>,
    parent: usize,
    target: Option<usize>,
) -> Option<ScriptEntitySet> {
    let effect = match effect {
        None => {
            warn!("Surface effect is not set");
            return None;
        }
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
        }
        Some(effect) => effect,
    };

    match effect.surface() {
        None => {
            warn!("Attempted to exec on_surface_round_elapsed on non-surface");
            return None;
        }
        Some((area_id, points)) => {
            targets.affected_points = points.iter().map(|p| (p.x, p.y)).collect();

            let area = GameState::get_area_state(area_id).unwrap();
            if let Some(target) = target {
                targets.indices.push(Some(target));
            } else {
                let inside = area.borrow().entities_with_points(points);
                targets.indices = inside.into_iter().map(Some).collect();
            }
        }
    }

    Some(targets)
}

impl UserData for CallbackData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
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

        methods.add_method_mut(
            "add_affected_points",
            |_, cb, points: Vec<HashMap<String, i32>>| {
                if let Kind::Script(_) = cb.kind {
                    warn!("Setting targets on global generated callback will have no effect");
                }

                cb.create_targets_if_missing();
                if let Some(ref mut cb_targets) = cb.targets {
                    for p in points {
                        let (x, y) = script_entity::unwrap_point(p)?;
                        cb_targets.affected_points.push((x, y));
                    }
                }

                Ok(())
            },
        );

        methods.add_method_mut("set_on_held_changed_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnHeldChanged, func);
            Ok(())
        });
        methods.add_method_mut("set_on_effect_applied_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnEffectApplied, func);
            Ok(())
        });
        methods.add_method_mut("set_on_menu_select_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnMenuSelect, func);
            Ok(())
        });
        methods.add_method_mut("set_on_removed_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnRemoved, func);
            Ok(())
        });
        methods.add_method_mut("set_on_damaged_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnDamaged, func);
            Ok(())
        });
        methods.add_method_mut("set_before_attack_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::BeforeAttack, func);
            Ok(())
        });
        methods.add_method_mut("set_after_attack_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::AfterAttack, func);
            Ok(())
        });
        methods.add_method_mut("set_before_defense_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::BeforeDefense, func);
            Ok(())
        });
        methods.add_method_mut("set_after_defense_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::AfterDefense, func);
            Ok(())
        });
        methods.add_method_mut("set_on_anim_update_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnAnimUpdate, func);
            Ok(())
        });
        methods.add_method_mut("set_on_anim_complete_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnAnimComplete, func);
            Ok(())
        });
        methods.add_method_mut("set_on_round_elapsed_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnRoundElapsed, func);
            Ok(())
        });
        methods.add_method_mut("set_on_moved_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnMoved, func);
            Ok(())
        });
        methods.add_method_mut("set_on_surface_round_elapsed_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnSurfaceRoundElapsed, func);
            Ok(())
        });
        methods.add_method_mut("set_on_moved_in_surface_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnMovedInSurface, func);
            Ok(())
        });
        methods.add_method_mut("set_on_entered_surface_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnEnteredSurface, func);
            Ok(())
        });
        methods.add_method_mut("set_on_exited_surface_fn", |_, cb, func: String| {
            cb.add_func(FuncKind::OnExitedSurface, func);
            Ok(())
        });
    }
}

/// ScriptHitKind stores the result of an attack for lua.  Includes the hit kind
/// and any damage.
///
/// # `is_miss() -> Bool`
/// Whether the attack was a miss
///
/// # `is_graze() -> Bool`
/// Whether the attack was a graze
///
/// # `is_hit() -> Bool`
/// Whether the attack was a hit
///
/// # `is_crit() -> Bool`
/// Whether the attack was a crit
///
/// # `total_damage() -> Int`
/// The total damage (in hit points) that the attack did.
///
/// # `damage_of_type(type: String) -> Int`
/// Returns the total damage (in hit points) from this attack for the given
/// damage type
///
/// # `entries() -> Table`
/// Creates a table of damage entries in this hit.  Iterating over the table
/// will allow you to access each damage type and corresponding amount.
/// ## Examples
/// ```lua
///   entries = hit:entries()
///   for i = 1, #entries do
///     game:log("Type: " .. entries[i]:type() .. ", amount: " .. tostring(entries[i]:amount()))
///   end
/// ```
///
/// # `kind() -> String`
/// The type of hit.  One of `Miss`, `Graze`, `Hit`, or `Crit`.
#[derive(Clone)]
pub struct ScriptHitKind {
    pub kind: HitKind,
    entries: Vec<DamageEntry>,
    pub total_damage: u32,
}

#[derive(Clone)]
struct DamageEntry {
    kind: &'static str,
    amount: u32,
}
impl UserData for DamageEntry {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("kind", |_, entry, ()| Ok(entry.kind));
        methods.add_method("amount", |_, entry, ()| Ok(entry.amount));
    }
}

impl ScriptHitKind {
    pub fn new(kind: HitKind, damage: Vec<(DamageKind, u32)>) -> ScriptHitKind {
        let mut total_damage = 0;
        let mut entries = Vec::new();
        for (kind, amount) in damage {
            total_damage += amount;
            entries.push(DamageEntry {
                kind: kind.to_str(),
                amount,
            });
        }

        ScriptHitKind {
            kind,
            entries,
            total_damage,
        }
    }
}

impl UserData for ScriptHitKind {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("is_miss", |_, hit, ()| Ok(hit.kind == HitKind::Miss));
        methods.add_method("is_graze", |_, hit, ()| Ok(hit.kind == HitKind::Graze));
        methods.add_method("is_hit", |_, hit, ()| Ok(hit.kind == HitKind::Hit));
        methods.add_method("is_crit", |_, hit, ()| Ok(hit.kind == HitKind::Crit));
        methods.add_method("total_damage", |_, hit, ()| Ok(hit.total_damage));
        methods.add_method("entries", |_, hit, ()| {
            let table = hit.entries.clone();
            Ok(table)
        });
        methods.add_method("damage_of_type", |_, hit, kind: String| {
            let mut total = 0;
            for entry in hit.entries.iter() {
                if entry.kind != kind {
                    continue;
                }
                total += entry.amount;
            }
            Ok(total)
        });
        methods.add_method("kind", |_, hit, ()| Ok(format!("{:?}", hit.kind)));
    }
}
