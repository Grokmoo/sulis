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

//! This module contains Sulis' scripting API.  Most structs are inserted into lua scripts as
//! objects.  The documentation for each struct describes the available functions on each
//! object when interacting with them within a lua script.
//!
//! There are currently four kinds of scripts:
//!
//! 1. AI Scripts:  These are attached to a given actor in their resource definition under `ai`.
//!    Whenever the parent entity is active, the `ai_action(parent, state)` method is called.
//! 2. Area / Trigger Scripts: These are called by triggers, conversations, and cutscenes.
//!    Named script functions are called, via a `fire_script` type containing an `id` for the
//!    script and a `func`.
//! 3. Ability Scripts: These are called when activating an ability or when an active ability
//!    meets certain conditions.  The entry point for the script is `on_activate(parent, ability)`.
//!    When using a targeter, `on_target_select(parent, ability, targets)` is the return from that
//!    targeter.
//! 4. Item Scripts: Similar to ability scripts, but called when using an item.  The entry point is
//!    `on_activate(parent, item)`.
//!
//! Since standard Lua methods for referencing other script files will not work, Sulis includes
//! a simple facility to include the contents of a script into another script.  This is done
//! with a line in the target script:
//! --INCLUDE <other_script_id>
//! The entire contents of the referenced script will be placed into the script file in place
//! of the --INCLUDE directive at evaluation time.
//! --INCLUDE directives are only evaluated one level deep at this time.

mod area_targeter;
pub use self::area_targeter::AreaTargeter;

mod module_export;
pub use self::module_export::ModuleExport;

mod script_ability;
pub use self::script_ability::{ScriptAbility, ScriptAbilitySet};

pub mod script_cache;

pub mod script_callback;
pub use self::script_callback::{
    CallbackData, FuncKind, ScriptCallback, ScriptHitKind, TriggeredCallback,
};

mod script_effect;
pub use self::script_effect::{
    ScriptActiveSurface, ScriptAppliedEffect, ScriptEffect, ScriptMenuSelection,
};

mod script_entity;
pub use self::script_entity::ScriptEntity;

mod script_entity_set;
pub use self::script_entity_set::ScriptEntitySet;

mod script_interface;
pub use self::script_interface::{entity_with_id, ScriptInterface};

mod script_inventory;
pub use self::script_inventory::{ScriptInventory, ScriptStashItem, ScriptUsableItem};

mod script_item;
pub use self::script_item::{ScriptItem, ScriptItemKind};

mod script_menu;
pub use self::script_menu::ScriptMenu;

mod script_color_animation;
pub use self::script_color_animation::ScriptColorAnimation;

mod script_image_layer_animation;
pub use self::script_image_layer_animation::ScriptImageLayerAnimation;

mod script_particle_generator;
pub use self::script_particle_generator::ScriptParticleGenerator;

mod script_scale_animation;
pub use self::script_scale_animation::ScriptScaleAnimation;

mod script_subpos_animation;
pub use self::script_subpos_animation::ScriptSubposAnimation;

pub mod targeter;
pub use self::targeter::TargeterData;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time;

use rlua::{self, FromLuaMulti, Function, Lua, ToLuaMulti};

use crate::{ai, EntityState, GameState};
use sulis_core::{config::Config, util::Point};
use sulis_module::{Ability, DamageKind, HitKind, Module, QuickSlot};

pub type Result<T> = std::result::Result<T, rlua::Error>;

/// Script Helper module for easily calling various script methods
pub struct Script {}

impl Script {
    pub fn ai(parent: &Rc<RefCell<EntityState>>, func: &str) -> ai::State {
        match script_cache::ai_script(parent, func) {
            Err(e) => {
                warn!("Error in lua AI script: '{}'", e);
                ai::State::End
            }
            Ok(val) => val,
        }
    }

    pub fn entity(parent: &Rc<RefCell<EntityState>>, targets: ScriptEntitySet, func: &str) {
        let t: Option<usize> = None;
        if let Err(e) = script_cache::entity_script(parent, targets, t, func) {
            warn!("Error in entity script '{}': {}", func, e);
        }
    }

    pub fn entity_with_attack_data(
        parent: &Rc<RefCell<EntityState>>,
        targets: ScriptEntitySet,
        kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
        func: &str,
    ) {
        let t = Some(ScriptHitKind::new(kind, damage));
        if let Err(e) = script_cache::entity_script(parent, targets, t, func) {
            warn!("Error in entity with attack data script '{}': {}", func, e);
        }
    }

    pub fn entity_with_arg<T>(
        parent: &Rc<RefCell<EntityState>>,
        targets: ScriptEntitySet,
        arg: T,
        func: &str,
    ) where
        T: rlua::UserData + Send + 'static,
    {
        if let Err(e) = script_cache::entity_script(parent, targets, Some(arg), func) {
            warn!("Error in entity with arg script '{}': {}", func, e);
        }
    }

    pub fn item_on_activate(parent: &Rc<RefCell<EntityState>>, func: String, kind: ScriptItemKind) {
        if let Err(e) = script_cache::item_on_activate(parent, func, kind) {
            warn!("Error in item on_activate script: {}", e);
        }
    }

    pub fn item(
        parent: &Rc<RefCell<EntityState>>,
        kind: ScriptItemKind,
        targets: ScriptEntitySet,
        func: &str,
    ) {
        let t: Option<usize> = None;
        if let Err(e) = script_cache::item_script(parent, kind, targets, t, func) {
            warn!("Error in item script '{}': {}", func, e);
        }
    }

    pub fn item_with_attack_data(
        parent: &Rc<RefCell<EntityState>>,
        i_kind: ScriptItemKind,
        targets: ScriptEntitySet,
        kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
        func: &str,
    ) {
        let t = Some(ScriptHitKind::new(kind, damage));
        if let Err(e) = script_cache::item_script(parent, i_kind, targets, t, func) {
            warn!("Error in item with attack data script '{}': {}", func, e);
        }
    }

    pub fn item_with_arg<T>(
        parent: &Rc<RefCell<EntityState>>,
        i_kind: ScriptItemKind,
        targets: ScriptEntitySet,
        arg: T,
        func: &str,
    ) where
        T: rlua::UserData + Send + 'static,
    {
        if let Err(e) = script_cache::item_script(parent, i_kind, targets, Some(arg), func) {
            warn!("Error in item with arg script '{}': {}", func, e);
        }
    }

    pub fn item_on_target_select(
        parent: &Rc<RefCell<EntityState>>,
        kind: ScriptItemKind,
        targets: Vec<Option<Rc<RefCell<EntityState>>>>,
        selected_point: Point,
        affected_points: Vec<Point>,
        func: &str,
        custom_target: Option<Rc<RefCell<EntityState>>>,
    ) {
        if let Err(e) = script_cache::item_on_target_select(
            parent,
            kind,
            targets,
            selected_point,
            affected_points,
            func,
            custom_target,
        ) {
            warn!("Error in item on target select: {}", e);
        }
    }

    pub fn ability_on_deactivate(parent: usize, ability: &Rc<Ability>) {
        if let Err(e) = script_cache::ability_on_deactivate(parent, ability) {
            warn!("Error in ability on_deactivate: {}", e);
        }
    }

    pub fn ability_on_activate(parent: usize, func: String, ability: &Rc<Ability>) {
        if let Err(e) = script_cache::ability_on_activate(parent, func, ability) {
            warn!("Error in ability on_activate: {}", e);
        }
    }

    pub fn ability_on_target_select(
        parent: &Rc<RefCell<EntityState>>,
        ability: &Rc<Ability>,
        targets: Vec<Option<Rc<RefCell<EntityState>>>>,
        selected_point: Point,
        affected_points: Vec<Point>,
        func: &str,
        custom_target: Option<Rc<RefCell<EntityState>>>,
    ) {
        if let Err(e) = script_cache::ability_on_target_select(
            parent,
            ability,
            targets,
            selected_point,
            affected_points,
            func,
            custom_target,
        ) {
            warn!("Error in ability on target select '{}': {}", func, e);
        }
    }

    pub fn ability_with_attack_data(
        parent: &Rc<RefCell<EntityState>>,
        ability: &Rc<Ability>,
        targets: ScriptEntitySet,
        kind: HitKind,
        damage: Vec<(DamageKind, u32)>,
        func: &str,
    ) {
        let t = Some(ScriptHitKind::new(kind, damage));
        if let Err(e) = script_cache::ability_script(parent, ability, targets, t, func) {
            warn!("Error in ability script '{}': {}", func, e);
        }
    }

    pub fn ability_with_arg<T>(
        parent: &Rc<RefCell<EntityState>>,
        ability: &Rc<Ability>,
        targets: ScriptEntitySet,
        arg: T,
        func: &str,
    ) where
        T: rlua::UserData + Send + 'static,
    {
        if let Err(e) = script_cache::ability_script(parent, ability, targets, Some(arg), func) {
            warn!("Error in ability script with arg '{}': {}", func, e);
        }
    }

    pub fn ability(
        parent: &Rc<RefCell<EntityState>>,
        ability: &Rc<Ability>,
        targets: ScriptEntitySet,
        func: &str,
    ) {
        let t: Option<usize> = None;
        if let Err(e) = script_cache::ability_script(parent, ability, targets, t, func) {
            warn!("Error in ability script '{}': {}", func, e);
        }
    }

    pub fn trigger<Arg>(script_id: &str, func: &str, arg: Arg)
    where
        Arg: for<'a> ToLuaMulti<'a>,
    {
        if let Err(e) = script_cache::trigger_script(script_id, func, arg) {
            warn!("Error in trigger script '{}/{}': {}", script_id, func, e);
        }
    }
}

const MEM_LIMIT: usize = 10_485_760;
const INSTRUCTION_LIMIT: u32 = 50_000;
const INSTRUCTIONS_PER_CHECK: u32 = 50;
const MILLIS_LIMIT: f64 = 50.0;

pub struct InstructionState {
    count: u32,
    start_time: time::Instant,
}

/// A script state, containing a complete lua state.
pub struct ScriptState {
    lua: Lua,
    id: String,
    instructions: Arc<Mutex<InstructionState>>,
    current_depth: Cell<u32>,
}

impl Default for ScriptState {
    fn default() -> ScriptState {
        let lua = Lua::new_with(get_rlua_std_lib());
        lua.set_memory_limit(Some(MEM_LIMIT));

        lua.context(|lua| {
            let globals = lua.globals();
            match globals.set("game", ScriptInterface {}) {
                Ok(()) => (),
                Err(e) => {
                    warn!("Error setting up Lua globals");
                    warn!("{}", e);
                }
            }
        });

        let instructions = Arc::new(Mutex::new(InstructionState {
            count: 0,
            start_time: time::Instant::now(),
        }));
        let state = ScriptState {
            lua,
            instructions,
            id: String::new(),
            current_depth: Cell::new(0),
        };

        let instructions = Arc::clone(&state.instructions);
        state.lua.set_hook(
            rlua::HookTriggers {
                every_nth_instruction: Some(INSTRUCTIONS_PER_CHECK),
                ..Default::default()
            },
            move |_, _| {
                let state = &mut *instructions.lock().unwrap();
                state.count += INSTRUCTIONS_PER_CHECK;

                if state.count > INSTRUCTION_LIMIT {
                    return Err(rlua::Error::RuntimeError(format!(
                        "Instruction limit of \
                         {INSTRUCTION_LIMIT} reached"
                    )));
                }

                if get_elapsed_millis(state.start_time.elapsed()) > MILLIS_LIMIT {
                    return Err(rlua::Error::RuntimeError(format!(
                        "Script time limit of \
                         {MILLIS_LIMIT} millis reached"
                    )));
                }

                Ok(())
            },
        );

        state
    }
}

impl ScriptState {
    fn print_report(&self, func: &str) {
        let (count, time) = {
            let inst = &(*self.instructions.lock().unwrap());
            (inst.count, inst.start_time.elapsed())
        };
        let total = get_elapsed_millis(time);
        let mem = (self.lua.used_memory() as f32) / 1024.0;
        log!(Config::bench_log_level(),
            "BENCHMARK Lua '{}:{}': {:.3} millis, {:.3} KB, ~{} Instructions",
            self.id, func, total, mem, count
        );
    }

    fn reset_instruction_state(&self) {
        let instructions = &mut *self.instructions.lock().unwrap();
        instructions.count = 0;
        instructions.start_time = time::Instant::now();
    }

    pub(in crate::script) fn load(&mut self, id: &str, script: &str) -> Result<()> {
        self.id = id.to_string();
        self.lua
            .context(|lua| lua.load(&script).set_name(&id)?.exec())
    }

    pub(in crate::script) fn exec_func<Args, Ret>(
        &self,
        function: &str,
        args: Args,
        report: bool,
    ) -> Result<Ret>
    where
        Args: for<'a> ToLuaMulti<'a>,
        Ret: for<'a> FromLuaMulti<'a>,
    {
        let cur_depth = self.current_depth.get();
        if cur_depth == 0 {
            self.reset_instruction_state();
        }
        self.current_depth.set(cur_depth + 1);
        if report {
            debug!(
                "Exec '{}:{}' with depth of {}",
                self.id,
                function,
                cur_depth + 1
            );
        }
        let result = self.lua.context(|lua| {
            let func: Function = lua.globals().get(function)?;
            func.call(args)
        });
        self.lua.gc_collect()?;
        if report {
            self.print_report(function);
        }
        self.current_depth.set(cur_depth);
        result
    }

    pub fn console(&self, script: String, party: &[Rc<RefCell<EntityState>>]) -> Result<String> {
        assert!(!party.is_empty());
        self.reset_instruction_state();
        let result = self.lua.context(|lua| {
            lua.globals().set("player", ScriptEntity::from(&party[0]))?;

            let party_table = lua.create_table()?;
            for (index, member) in party.iter().enumerate() {
                party_table.set(index + 1, ScriptEntity::from(member))?;
            }

            lua.globals().set("party", party_table)?;

            lua.load(&script).eval::<String>()
        });
        self.print_report("console");
        result
    }
}

fn get_targeter() -> Result<Rc<RefCell<AreaTargeter>>> {
    let area_state = GameState::area_state();
    let area_state = area_state.borrow();
    match area_state.targeter() {
        None => {
            warn!("Error getting targeter");
            Err(rlua::Error::ToLuaConversionError {
                from: "Lua",
                to: "Targeter",
                message: Some("No targeter is present".to_string()),
            })
        }
        Some(tg) => Ok(tg),
    }
}

fn get_rlua_std_lib() -> rlua::StdLib {
    use rlua::StdLib;

    StdLib::BASE | StdLib::TABLE | StdLib::STRING | StdLib::MATH
}

fn get_elapsed_millis(elapsed: time::Duration) -> f64 {
    (elapsed.as_secs() as f64) * 1000.0 + (elapsed.subsec_nanos() as f64) / 1_000_000.0
}
