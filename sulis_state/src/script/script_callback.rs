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

use sulis_module::Module;
use script::{ScriptEntity, ScriptEntitySet};
use GameState;

pub trait ScriptCallback {
    fn call(&self);
}

#[derive(Clone)]
pub struct CallbackData {
    parent: usize,
    ability_id: String,
    targets: Vec<usize>,
    func: String,
}

impl CallbackData {
    pub fn new(parent: usize, ability_id: &str, func: &str) -> CallbackData {
        CallbackData {
            parent,
            ability_id: ability_id.to_string(),
            targets: Vec::new(),
            func: func.to_string(),
        }
    }
}

impl ScriptCallback for CallbackData {
    fn call(&self) {
        let area_state = GameState::area_state();
        let parent = area_state.borrow().get_entity(self.parent);
        let ability = Module::ability(&self.ability_id).unwrap();
        let targets = self.targets.iter().map(|t| area_state.borrow().get_entity(*t)).collect();

        GameState::execute_ability_script(&parent, &ability, targets, &self.func);
    }
}

impl UserData for CallbackData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("add_target", |_, cb, target: ScriptEntity| {
            cb.targets.push(target.index);
            Ok(())
        });

        methods.add_method_mut("add_targets", |_, cb, targets: ScriptEntitySet| {
            cb.targets.append(&mut targets.indices.clone());
            Ok(())
        });
    }
}
