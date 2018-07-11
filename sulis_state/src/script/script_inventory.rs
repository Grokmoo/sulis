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

use sulis_rules::{Slot, WeaponStyle};
use ItemState;
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
                        Some(ref equippable) => equippable.attack.is_none(),
                    }
                }
            })
        });

        methods.add_method("weapon_style", |_, data, ()| {
            try_unwrap!(data => inv);

            Ok(ScriptWeaponStyle { style: inv.weapon_style() })
        });

        methods.add_method("add_coins", |_, data, amount: i32| {
            let parent = data.parent.try_unwrap()?;
            parent.borrow_mut().actor.add_coins(amount);
            Ok(())
        });

        methods.add_method("add_item", |_, data, item: String| {
            let parent = data.parent.try_unwrap()?;
            let item = match ItemState::from(&item) {
                None => return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "Item",
                    message: Some(format!("Item '{}' does not exist", item)),
                }),
                Some(item) => item,
            };
            parent.borrow_mut().actor.add_item(item);
            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct ScriptWeaponStyle {
    pub style: WeaponStyle,
}

impl UserData for ScriptWeaponStyle {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("to_string", |_, style, ()| {
            use sulis_rules::WeaponStyle::*;
            Ok(match style.style {
                Ranged => "Ranged",
                TwoHanded => "TwoHanded",
                Single => "Single",
                Shielded => "Shielded",
                DualWielding => "DualWielding",
            }.to_string())
        });
    }
}
