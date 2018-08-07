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

use sulis_rules::{Slot, QuickSlot};
use sulis_module::ability::AIData;
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

            Ok(match inv.equipped(Slot::HeldMain) {
                None => false,
                Some(_) => true,
            })
        });

        methods.add_method("has_equipped_shield", |_, data, ()| {
            try_unwrap!(data => inv);

            Ok(match inv.equipped(Slot::HeldOff) {
                None => false,
                Some(ref item_state) => {
                    match item_state.item.equippable {
                        None => false,
                        Some(ref equippable) => equippable.attack.is_none(),
                    }
                }
            })
        });

        methods.add_method("has_alt_weapons", |_, data, ()| {
            try_unwrap!(data => inv);

            let result = inv.quick(QuickSlot::AltHeldMain).is_some() ||
                inv.quick(QuickSlot::AltHeldOff).is_some();
            Ok(result)
        });

        methods.add_method("alt_weapon_style", |_, data, ()| {
            try_unwrap!(data => inv);
            Ok(format!("{:?}", inv.alt_weapon_style()))
        });

        methods.add_method("weapon_style", |_, data, ()| {
            try_unwrap!(data => inv);
            Ok(format!("{:?}", inv.weapon_style()))
        });

        methods.add_method("usable_items", |_, data, ()| {
            let parent = data.parent.try_unwrap()?;
            let parent = parent.borrow();
            let mut items = Vec::new();
            for slot in QuickSlot::usable_iter() {
                let item = match parent.actor.inventory().quick(*slot) {
                    None => continue,
                    Some(item) => item,
                };

                let usable = match &item.item.usable {
                    None => unreachable!(),
                    Some(usable) => usable,
                };

                items.push(ScriptUsableItem {
                    parent: data.parent.clone(),
                    slot: *slot,
                    ai: usable.ai,
                });
            }

            Ok(items)
        });
    }
}

#[derive(Clone)]
pub struct ScriptUsableItem {
    parent: ScriptEntity,
    pub slot: QuickSlot,
    ai: AIData,
}

impl UserData for ScriptUsableItem {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("ai_data", |lua, item, ()| {
            let ai_data = lua.create_table()?;
            ai_data.set("priority", item.ai.priority())?;
            ai_data.set("kind", item.ai.kind())?;
            ai_data.set("group", item.ai.group())?;
            ai_data.set("range", item.ai.range())?;

            Ok(ai_data)
        });
    }
}
