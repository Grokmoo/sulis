//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::collections::HashMap;

use rlua::{self, Context, UserData, UserDataMethods};

use sulis_module::{on_trigger, OnTrigger};
use crate::GameState;
use crate::script::ScriptEntity;

/// A data structure representing all data that will be
/// transfered from this module to the specified subsequent module.
/// This is created with `game:create_module_export(module_id)`.
/// It is assumed the player character will always be exported.
///
/// # `activate()`
/// Actives this export function, triggering the loading of the new campaign.
///
/// # `set_include_stash(bool)`
/// Sets whether the party stash will be exported along with the player.
/// Defaults to true.
///
/// # `set_flag(flag: String, value: String (Optional))`
/// Sets a `flag` which will be stored on the player entity in the newly
/// loaded campaign.  If the value is not specified, sets that the flag
/// exists but does not neccesarily set a specific value.  This allows
/// arbitrary data to be passed between campaigns.
///
/// # `add_to_party(entity: ScriptEntity)`
/// Adds the specified `ScriptEntity` to the player's party in the next
/// campaign.
///
#[derive(Clone)]
pub struct ModuleExport {
    module: String,
    include_stash: bool,
    party: Vec<ScriptEntity>,
    custom_flags: HashMap<String, String>,
}

impl ModuleExport {
    pub fn new(module: String) -> ModuleExport {
        ModuleExport {
            module,
            include_stash: true,
            party: Vec::new(),
            custom_flags: HashMap::new(),
        }
    }
}

impl UserData for ModuleExport {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", &activate);

        methods.add_method_mut("set_include_stash", |_, export, include: bool| {
            export.include_stash = include;
            Ok(())
        });

        methods.add_method_mut("set_flag", |_, export, (flag, value): (String, Option<String>)| {
            let val = value.unwrap_or("true".to_string());
            export.custom_flags.insert(flag, val);
            Ok(())
        });

        methods.add_method_mut("add_to_party", |_, export, entity: ScriptEntity| {
            export.party.push(entity);
            Ok(())
        });
    }
}

fn activate(_lua: Context, export: &ModuleExport, _: ()) -> Result<(), rlua::Error> {
    let data = on_trigger::ModuleLoadData {
        module: export.module.to_string(),
        include_stash: export.include_stash,
        party: export.party.iter().map(|e| e.index).flatten().collect(),
        flags: export.custom_flags.clone(),
    };
    let pc = GameState::player();
    let cb = OnTrigger::LoadModule(data);

    GameState::add_ui_callback(vec![cb], &pc, &pc);
    Ok(())
}
