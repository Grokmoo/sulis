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

mod area_targeter;
use self::area_targeter::AreaTargeter;

pub mod script_callback;
use self::script_callback::CallbackData;
pub use self::script_callback::ScriptCallback;
pub use self::script_callback::ScriptHitKind;

mod script_effect;
use self::script_effect::ScriptEffect;

mod script_entity;
pub use self::script_entity::ScriptEntity;
pub use self::script_entity::ScriptEntitySet;

mod script_inventory;
pub use self::script_inventory::ScriptInventory;

mod script_color_animation;
use self::script_color_animation::ScriptColorAnimation;

mod script_particle_generator;
use self::script_particle_generator::ScriptParticleGenerator;

mod script_subpos_animation;
use self::script_subpos_animation::ScriptSubposAnimation;

pub mod targeter;
use self::targeter::Targeter;
use self::targeter::TargeterData;

use std;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{self, Function, Lua, UserData, UserDataMethods};

use sulis_core::config::CONFIG;
use sulis_core::util::Point;
use sulis_module::{ability, Ability, Module, OnTrigger};
use {EntityState, GameState};

type Result<T> = std::result::Result<T, rlua::Error>;

pub struct ScriptState {
    lua: Lua,
}

impl ScriptState {
    pub fn new() -> ScriptState {
        let lua = Lua::new();

        {
            let globals = lua.globals();
            match globals.set("game", ScriptInterface {}) {
                Ok(()) => (),
                Err(e) => {
                    warn!("Error setting up Lua globals");
                    warn!("{}", e);
                },
            }
        }

        ScriptState { lua }
    }

    fn execute_script(&self, parent: &Rc<RefCell<EntityState>>, function_args: &str,
                          mut script: String, function: &str) -> Result<()> {
        let globals = self.lua.globals();
        globals.set("parent", ScriptEntity::from(&parent))?;

        debug!("Loading script for '{}'", function);

        script.push('\n');
        script.push_str(function);
        script.push_str(function_args);
        let func: Function = self.lua.load(&script, Some(function))?;

        debug!("Calling script function '{}'", function);

        func.call::<_, ()>("")?;

        Ok(())
    }

    pub fn console(&self, script: String, party: &Vec<Rc<RefCell<EntityState>>>) -> Result<String> {
        assert!(party.len() > 0);
        self.lua.globals().set("player", ScriptEntity::from(&party[0]))?;

        let party_table = self.lua.create_table()?;
        for (index, member) in party.iter().enumerate() {
            party_table.set(index + 1, ScriptEntity::from(member))?;
        }

        self.lua.globals().set("party", party_table)?;

        self.lua.eval::<String>(&script, None)
    }

    pub fn ability_on_activate(&self, parent: &Rc<RefCell<EntityState>>,
                                       ability: &Rc<Ability>) -> Result<()> {
        let t: Option<(&str, usize)> = None;
        self.ability_script(parent, ability, ScriptEntitySet::new(parent, &Vec::new()), t, "on_activate")
    }

    pub fn ability_on_target_select(&self,
                                    parent: &Rc<RefCell<EntityState>>,
                                    ability: &Rc<Ability>,
                                    targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                                    selected_point: Point,
                                    affected_points: Vec<Point>,
                                    func: &str,
                                    custom_target: Option<Rc<RefCell<EntityState>>>) -> Result<()> {
        let mut targets = ScriptEntitySet::new(parent, &targets);
        targets.selected_point = Some((selected_point.x, selected_point.y));
        let arg = match custom_target {
            None => None,
            Some(entity) => Some(("custom_target", ScriptEntity::from(&entity))),
        };
        targets.affected_points = affected_points.into_iter().map(|p| (p.x, p.y)).collect();
        self.ability_script(parent, ability, targets, arg, func)
    }

    pub fn ability_script<'a, T>(&'a self, parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                 targets: ScriptEntitySet, arg: Option<(&str, T)>,
                                 func: &str) -> Result<()> where T: rlua::prelude::ToLua<'a> + Send {
        let script = get_script(ability)?;
        self.lua.globals().set("ability", ScriptAbility::from(ability))?;
        self.lua.globals().set("targets", targets)?;

        let mut args_string = "(parent, ability, targets".to_string();
        if let Some((arg_str, arg)) = arg {
            args_string.push_str(", ");
            args_string.push_str(arg_str);

            self.lua.globals().set(arg_str, arg)?;
        }
        args_string.push(')');
        self.execute_script(parent, &args_string, script.to_string(), func)
    }

    pub fn trigger_script(&self, script_id: &str, func: &str, parent: &Rc<RefCell<EntityState>>,
                          target: &Rc<RefCell<EntityState>>) -> Result<()> {
        let script = get_script_from_id(script_id)?;
        self.lua.globals().set("target", ScriptEntity::from(target))?;

        self.execute_script(parent, "(parent, target)", script, func)
    }
}

fn get_script_from_id(id: &str) -> Result<String> {
    match Module::script(id) {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "&str",
            to: "Script",
            message: Some(format!("No script found with id '{}'", id).to_string()),
        }),
        Some(ref script) => Ok(script.to_string())
    }
}

fn get_script(ability: &Rc<Ability>) -> Result<&str> {
    match ability.active {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "Rc<Ability>",
            to: "ScriptAbility",
            message: Some("The Ability is not active".to_string()),
        }),
        Some(ref active) => Ok(&active.script),
    }
}

struct ScriptInterface { }

impl UserData for ScriptInterface {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("log", |_, _, val: String| {
            info!("[LUA]: {}", val);
            Ok(())
        });

        methods.add_method("anim_base_time", |_, _, ()| {
            let secs = CONFIG.display.animation_base_time_millis as f32 / 1000.0;
            Ok(secs)
        });

        methods.add_method("atan2", |_, _, (x, y): (f32, f32)| {
            Ok(y.atan2(x))
        });

        methods.add_method("is_passable", |_, _, (entity, x, y): (ScriptEntity, i32, i32)| {
            let area_state = GameState::area_state();
            let area_state = area_state.borrow();
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            let entities_to_ignore = vec![entity.index];
            Ok(area_state.is_passable(&entity, &entities_to_ignore, x, y))
        });

        methods.add_method("spawn_encounter_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();

            if !area_state.spawn_encounter_at(x, y) {
                warn!("Unable to find encounter for script spawn at {},{}", x, y);
            }

            let mgr = GameState::turn_manager();
            mgr.borrow_mut().check_ai_activation_for_party(&area_state);

            Ok(())
        });

        methods.add_method("enable_trigger_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_trigger_enabled_at(x, y, true) {
                warn!("Unable to find trigger at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("disable_trigger_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_trigger_enabled_at(x, y, false) {
                warn!("Unable to find trigger at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("enable_prop_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_prop_enabled_at(x, y, true) {
                warn!("Unable to find prop at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("disable_prop_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_prop_enabled_at(x, y, false) {
                warn!("Unable to find prop at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("say_line", |_, _, (line, target): (String, Option<ScriptEntity>)| {
            let pc = GameState::player();
            let target = match target {
                None => Rc::clone(&pc),
                Some(ref entity) => {
                    entity.try_unwrap()?
                }
            };

            let cb = OnTrigger {
                say_line: Some(line),
                ..Default::default()
            };

            GameState::add_ui_callback(cb, &pc, &target);
            Ok(())
        });

        methods.add_method("start_conversation", |_, _, (id, target): (String, Option<ScriptEntity>)| {
            let pc = GameState::player();
            let target = match target {
                None => Rc::clone(&pc),
                Some(ref entity) => {
                    entity.try_unwrap()?
                }
            };

            let cb = OnTrigger {
                start_conversation: Some(id),
                ..Default::default()
            };

            GameState::add_ui_callback(cb, &pc, &target);
            Ok(())
        });

        methods.add_method("entities_with_ids", |_, _, ids: Vec<String>| {
            Ok(entities_with_ids(ids))
        });

        methods.add_method("entity_with_id", |_, _, id: String| {
            match entity_with_id(id) {
                Some(entity) => Ok(ScriptEntity::from(&entity)),
                None => Err(rlua::Error::ToLuaConversionError {
                    from: "ID",
                    to: "ScriptEntity",
                    message: Some("Target with the specified ID does not exist".to_string())
                })
            }
        });

        methods.add_method("add_party_member", |_, _, id: String| {
            let entity = match entity_with_id(id) {
                Some(entity) => entity,
                None => return Err(rlua::Error::ToLuaConversionError {
                    from: "ID",
                    to: "ScriptENtity",
                    message: Some("Entity with specified id does not exist".to_string())
                })
            };

            GameState::add_party_member(entity);

            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct ScriptAbility {
    id: String,
    name: String,
    duration: u32,
    ap: u32,
}

impl ScriptAbility {
    fn from(ability: &Rc<Ability>) -> ScriptAbility {
        let duration = match ability.active {
            None => 0,
            Some(ref active) => match active.duration {
                ability::Duration::Rounds(rounds) => rounds,
                ability::Duration::Mode => 0,
                ability::Duration::Instant => 0,
            }
        };

        let ap = match ability.active {
            None => 0,
            Some(ref active) => active.ap,
        };

        ScriptAbility {
            id: ability.id.to_string(),
            name: ability.name.to_string(),
            duration,
            ap,
        }
    }

    fn error_if_not_active(&self) -> Result<()> {
        let ability = match Module::ability(&self.id) {
            None => unreachable!(),
            Some(ability) => ability,
        };

        match ability.active {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptAbility",
                to: "ActiveAbility",
                message: Some(format!("The ability '{}' is not active", self.id)),
            }),
            Some(_) => Ok(())
        }
    }
}

impl UserData for ScriptAbility {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("is_active_mode", |_, ability, target: ScriptEntity| {
            ability.error_if_not_active()?;
            let target = target.try_unwrap()?;
            let mut target = target.borrow_mut();
            match target.actor.ability_state(&ability.id) {
                None => Ok(false),
                Some(ref ability_state) => Ok(ability_state.is_active_mode()),
            }
        });
        methods.add_method("activate", &activate);
        methods.add_method("deactivate", |_, ability, target: ScriptEntity| {
            ability.error_if_not_active()?;
            let target = target.try_unwrap()?;
            target.borrow_mut().actor.deactivate_ability_state(&ability.id);
            Ok(())
        });
        methods.add_method("name", |_, ability, ()| {
            Ok(ability.name.to_string())
        });
        methods.add_method("duration", |_, ability, ()| Ok(ability.duration));

        methods.add_method("create_callback", |_, ability, parent: ScriptEntity| {
            ability.error_if_not_active()?;
            let index = parent.try_unwrap_index()?;
            let cb_data = CallbackData::new(index, &ability.id);
            Ok(cb_data)
        });
    }
}

fn entities_with_ids(ids: Vec<String>) -> Vec<ScriptEntity> {
    let mut result = Vec::new();

    let area_state = GameState::area_state();
    let area_id = area_state.borrow().area.id.to_string();

    let mgr = GameState::turn_manager();
    for entity in mgr.borrow().entity_iter() {
        {
            let entity = entity.borrow();
            if !entity.location.is_in_area_id(&area_id) { continue; }
            if !ids.contains(&entity.actor.actor.id) { continue; }
        }

        result.push(ScriptEntity::from(&entity));
    }

    result
}

fn entity_with_id(id: String) -> Option<Rc<RefCell<EntityState>>> {
    let area_state = GameState::area_state();
    let area_id = area_state.borrow().area.id.to_string();

    let mgr = GameState::turn_manager();
    for entity in mgr.borrow().entity_iter() {
        {
            let entity = entity.borrow();
            if !entity.location.is_in_area_id(&area_id) { continue; }
            if entity.actor.actor.id != id { continue; }
        }

        return Some(entity)
    }

    None
}

fn activate(_lua: &Lua, ability: &ScriptAbility, (target, take_ap): (ScriptEntity, Option<bool>)) -> Result<()> {
    ability.error_if_not_active()?;
    let entity = target.try_unwrap()?;
    let take_ap = take_ap.unwrap_or(true);

    let mgr = GameState::turn_manager();
    if take_ap && mgr.borrow().is_combat_active() {
        entity.borrow_mut().actor.remove_ap(ability.ap);
    }

    entity.borrow_mut().actor.activate_ability_state(&ability.id);

    Ok(())
}
