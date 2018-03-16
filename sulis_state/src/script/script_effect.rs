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

use rlua::{Lua, UserData, UserDataMethods};

use sulis_rules::{BonusList, Damage, DamageKind};

use script::Result;
use {Effect, GameState};

#[derive(Clone)]
pub struct ScriptEffect {
    parent: usize,
    name: String,
    duration: u32,
    pub bonuses: BonusList,
}

impl ScriptEffect {
    pub fn new(parent: usize, name: &str, duration: u32) -> ScriptEffect {
        ScriptEffect {
            parent,
            name: name.to_string(),
            duration,
            bonuses: BonusList::default(),
        }
    }
}

impl UserData for ScriptEffect {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("apply", &apply);
        methods.add_method_mut("add_num_bonus", &add_num_bonus);
        methods.add_method_mut("add_damage", |_, effect, (min, max): (u32, u32)| {
            effect.bonuses.bonus_damage = Some(Damage { min, max, kind: None });
            Ok(())
        });
        methods.add_method_mut("add_damage_of_kind", |_, effect, (min, max, kind): (u32, u32, String)| {
            let kind = DamageKind::from_str(&kind);
            effect.bonuses.bonus_damage = Some(Damage { min, max, kind: Some(kind) });
            Ok(())
        });
    }
}

fn add_num_bonus(_lua: &Lua, effect: &mut ScriptEffect, args: (String, f32)) -> Result<()> {
    let (name, amount) = args;
    let name = name.to_lowercase();
    let amount_int = amount as i32;

    trace!("Adding numeric bonus {} to '{}'", amount, name);
    match name.as_ref() {
        "armor" => effect.bonuses.base_armor = Some(amount as u32),
        "reach" => effect.bonuses.bonus_reach = Some(amount),
        "range" => effect.bonuses.bonus_range = Some(amount),
        "initiative" => effect.bonuses.initiative = Some(amount_int),
        "hit_points" => effect.bonuses.hit_points = Some(amount_int),
        "accuracy" => effect.bonuses.accuracy = Some(amount_int),
        "defense" => effect.bonuses.defense = Some(amount_int),
        "fortitude" => effect.bonuses.fortitude = Some(amount_int),
        "reflex" => effect.bonuses.reflex = Some(amount_int),
        "will" => effect.bonuses.will = Some(amount_int),
        _ => warn!("Attempted to add int bonus with invalid type '{}'", name),
    }
    Ok(())
}

const TURNS_TO_MILLIS: u32 = 5000;

fn apply(_lua: &Lua, effect_data: &ScriptEffect, _args: ()) -> Result<()> {
    let area_state = GameState::area_state();
    let area_state = area_state.borrow();

    let entity = area_state.get_entity(effect_data.parent);

    let duration = effect_data.duration * TURNS_TO_MILLIS;

    trace!("Apply effect to '{}'", entity.borrow().actor.actor.name);
    let effect = Effect::new(&effect_data.name, duration, effect_data.bonuses.clone());
    entity.borrow_mut().actor.add_effect(effect);

    Ok(())
}
