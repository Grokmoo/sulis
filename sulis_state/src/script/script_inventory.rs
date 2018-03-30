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

use sulis_module::item::Slot;
use script::*;

#[derive(Clone)]
pub struct ScriptInventory {
    parent: ScriptEntity,
}

impl ScriptInventory {
    pub fn new(parent: ScriptEntity) -> ScriptInventory {
        ScriptInventory { parent }
    }
}

macro_rules! try_unwrap {
    ($name:expr => $v:ident) => {
        let parent = $name.parent.try_unwrap()?;
        let parent = parent.borrow();
        let $v = &parent.actor.inventory();
    }
}

impl UserData for ScriptInventory {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("has_equipped_weapon", |_, data, ()| {
            try_unwrap!(data => inv);

            Ok(match inv.get(Slot::HeldMain) {
                None => false,
                Some(_) => true,
            })
        });

        methods.add_method("has_equipped_shield", |_, data, ()| {
            try_unwrap!(data => inv);

            Ok(match inv.get(Slot::HeldOff) {
                None => false,
                Some(ref item_state) => {
                    match item_state.item.equippable {
                        None => false,
                        Some(ref equippable) => equippable.bonuses.attack.is_none(),
                    }
                }
            })
        });
    }
}
