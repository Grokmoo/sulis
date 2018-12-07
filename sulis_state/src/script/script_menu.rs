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

use rlua::{UserData, UserDataMethods};

use sulis_module::on_trigger::{self, OnTrigger, ScriptMenuChoice};
use crate::script::{script_callback::FuncKind, CallbackData};
use crate::GameState;

/// A user interface menu being created by a script.  Normally created
/// by `game:create_menu()`
///
/// # `add_choice(text: String, value: String (Optional))`
/// Adds a choice to this menu that the user can select.  The `text` is
/// displayed.  The selection return is either `value` if specified,
/// or `text` if not.
///
/// # `show()`
/// Shows this menu, allowing the user to select their choice.
#[derive(Clone)]
pub struct ScriptMenu {
    title: String,
    choices: Vec<ScriptMenuChoice>,
    callback: CallbackData,
}

impl ScriptMenu {
    pub fn new(title: String, callback: CallbackData) -> ScriptMenu {
        ScriptMenu {
            title,
            choices: Vec::new(),
            callback,
        }
    }
}

impl UserData for ScriptMenu {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("add_choice", |_, menu, (text, value): (String, Option<String>)| {
            let value = match value {
                None => text.clone(),
                Some(value) => value,
            };

            menu.choices.push(ScriptMenuChoice { display: text, value });
            Ok(())
        });

        methods.add_method("show", |_, menu, ()| {
            let func = match menu.callback.get_func(FuncKind::OnMenuSelect) {
                None => return Err(rlua::Error::FromLuaConversionError {
                    from: "CallbackData",
                    to: "Menu",
                    message: Some("OnMenuSelect must be specified for callback".to_string())
                }),
                Some(func) => func,
            };

            let choices = menu.choices.clone();

            let data = on_trigger::MenuData {
                title: menu.title.to_string(),
                choices,
                cb_func: func,
                cb_kind: menu.callback.kind(),
                cb_parent: menu.callback.parent(),
            };

            let pc = GameState::player();
            let cb = OnTrigger::ShowMenu(data);
            GameState::add_ui_callback(vec![cb], &pc, &pc);

            Ok(())
        });
    }
}
