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

use std::collections::HashSet;

use rlua::{UserData, UserDataMethods};

use sulis_module::Module;
use script::{ScriptEntity, ScriptEntitySet};
use GameState;

const BEFORE_ATTACK: &str = "before_attack";
const AFTER_ATTACK: &str = "after_attack";
const ON_ANIM_COMPLETE: &str = "on_anim_complete";

pub trait ScriptCallback {
    fn before_attack(&self) { }

    fn after_attack(&self) { }

    fn on_anim_complete(&self) { }
}

#[derive(Clone)]
pub struct CallbackData {
    parent: usize,
    ability_id: String,
    targets: Vec<usize>,
    funcs: HashSet<String>,
}

impl CallbackData {
    pub fn new(parent: usize, ability_id: &str) -> CallbackData {
        CallbackData {
            parent,
            ability_id: ability_id.to_string(),
            targets: Vec::new(),
            funcs: HashSet::new(),
        }
    }

    fn do_callback(&self, func: &str) {
        let area_state = GameState::area_state();
        let parent = area_state.borrow().get_entity(self.parent);
        let ability = Module::ability(&self.ability_id).unwrap();
        let targets = self.targets.iter().map(|t| area_state.borrow().get_entity(*t)).collect();

        GameState::execute_ability_script(&parent, &ability, targets, &func);
    }
}

impl ScriptCallback for CallbackData {
    fn before_attack(&self) {
        if self.funcs.contains(BEFORE_ATTACK) {
            self.do_callback(BEFORE_ATTACK);
        }
    }

    fn after_attack(&self) {
        if self.funcs.contains(AFTER_ATTACK) {
            self.do_callback(AFTER_ATTACK);
        }
    }

    fn on_anim_complete(&self) {
        if self.funcs.contains(ON_ANIM_COMPLETE) {
            self.do_callback(ON_ANIM_COMPLETE);
        }
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

        methods.add_method_mut("register_fn", |_, cb, func: String| {
            cb.funcs.insert(func);
            Ok(())
        });
    }
}
