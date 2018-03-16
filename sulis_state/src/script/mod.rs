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

mod script_callback;
use self::script_callback::CallbackData;
pub use self::script_callback::ScriptCallback;

mod script_effect;
use self::script_effect::ScriptEffect;

mod script_entity;
use self::script_entity::ScriptEntity;
use self::script_entity::ScriptEntitySet;

pub mod targeter;
use self::targeter::TargeterData;

use std;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{self, Function, Lua, UserData, UserDataMethods};

use sulis_module::{Ability};
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
                          script: &str, function: &str) -> Result<()> {
        let globals = self.lua.globals();
        globals.set("parent", ScriptEntity::from(&parent))?;

        debug!("Loading script for '{}'", function);

        let script = format!("{}\n{}{}", script, function, function_args);
        let func: Function = self.lua.load(&script, Some(function))?;

        debug!("Calling script function '{}'", function);

        func.call::<_, ()>("")?;

        Ok(())
    }

    pub fn ability_on_activate(&self, parent: &Rc<RefCell<EntityState>>,
                                       ability: &Rc<Ability>) -> Result<()> {
        self.ability_script(parent, ability, Vec::new(), "on_activate")
    }

    pub fn ability_on_target_select(&self, parent: &Rc<RefCell<EntityState>>,
                                            ability: &Rc<Ability>,
                                            targets: Vec<Rc<RefCell<EntityState>>>) -> Result<()> {
        self.ability_script(parent, ability, targets, "on_target_select")
    }

    pub fn ability_script(&self, parent: &Rc<RefCell<EntityState>>,
                                  ability: &Rc<Ability>,
                                  targets: Vec<Rc<RefCell<EntityState>>>,
                                  func: &str) -> Result<()> {
        let script = get_script(ability)?;
        self.lua.globals().set("ability", ScriptAbility::from(ability))?;
        self.lua.globals().set("targets", ScriptEntitySet::new(parent, &targets))?;
        self.execute_script(parent, "(parent, ability, targets)", script, func)
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
        assert!(ability.active.is_some());

        ScriptAbility {
            id: ability.id.to_string(),
            name: ability.name.to_string(),
            duration: ability.active.as_ref().unwrap().duration,
            ap: ability.active.as_ref().unwrap().ap,
        }
    }
}

impl UserData for ScriptAbility {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method("name", |_, ability, ()| {
            Ok(ability.name.to_string())
        });
        methods.add_method("duration", |_, ability, ()| Ok(ability.duration));

        methods.add_method("create_callback", |_, ability, (parent, func): (ScriptEntity, String)| {
            let cb_data = CallbackData::new(parent.index, &ability.id, &func);
            Ok(cb_data)
        });
    }
}

fn activate(_lua: &Lua, ability: &ScriptAbility, target: ScriptEntity) -> Result<()> {
    let area_state = GameState::area_state();
    let entity = area_state.borrow().get_entity(target.index);

    if area_state.borrow().turn_timer.is_active() {
        entity.borrow_mut().actor.remove_ap(ability.ap);
    }

    entity.borrow_mut().actor.activate_ability_state(&ability.id);

    Ok(())
}
