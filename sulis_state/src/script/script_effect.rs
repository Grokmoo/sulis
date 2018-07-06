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

use std::rc::Rc;

use rlua::{Lua, UserData, UserDataMethods};

use sulis_rules::{Attribute, Bonus, BonusList, Damage, DamageKind};

use script::{CallbackData, Result, script_particle_generator, ScriptParticleGenerator,
    script_color_animation, ScriptColorAnimation, ScriptAbility};
use {Effect, GameState};

#[derive(Clone)]
pub struct ScriptEffect {
    parent: usize,
    name: String,
    duration: u32,
    deactivate_with_ability: Option<String>,
    pub bonuses: BonusList,
    callbacks: Vec<CallbackData>,
}

impl ScriptEffect {
    pub fn new(parent: usize, name: &str, duration: u32) -> ScriptEffect {
        ScriptEffect {
            parent,
            name: name.to_string(),
            deactivate_with_ability: None,
            duration,
            bonuses: BonusList::default(),
            callbacks: Vec::new(),
        }
    }
}

impl UserData for ScriptEffect {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        // TODO refactor ScriptParticleGenerator, ScriptColorAnimation, and ScriptSubposAnimation
        // to all implement a common trait
        methods.add_method("apply_with_color_anim", |_, effect, anim: Option<ScriptColorAnimation>| {
            apply(effect, None, anim)
        });
        methods.add_method("apply", |_, effect, pgen: Option<ScriptParticleGenerator>| {
            apply(effect, pgen, None)
        });
        methods.add_method_mut("add_num_bonus", &add_num_bonus);
        methods.add_method_mut("add_damage", |_, effect, (min, max, ap): (u32, u32, Option<u32>)| {
            effect.bonuses.add_entry(Bonus::Damage(Damage { min, max, ap: ap.unwrap_or(0), kind: None }));
            Ok(())
        });
        methods.add_method_mut("add_move_disabled", |_, effect, ()| {
            effect.bonuses.add_entry(Bonus::MoveDisabled);
            Ok(())
        });
        methods.add_method_mut("add_attack_disabled", |_, effect, ()| {
            effect.bonuses.add_entry(Bonus::AttackDisabled);
            Ok(())
        });
        methods.add_method_mut("add_damage_of_kind", |_, effect, (min, max, kind, ap):
                               (u32, u32, String, Option<u32>)| {
            let kind = DamageKind::from_str(&kind);
            effect.bonuses.add_entry(Bonus::Damage(Damage { min, max, ap: ap.unwrap_or(0), kind: Some(kind) }));
            Ok(())
        });
        methods.add_method_mut("add_armor_of_kind", |_, effect, (value, kind): (i32, String)| {
            let kind = DamageKind::from_str(&kind);
            effect.bonuses.add_entry(Bonus::ArmorKind { kind, amount: value });
            Ok(())
        });
        methods.add_method_mut("add_attribute_bonus", |_, effect, (attr, amount): (String, i8)| {
            let attribute = match Attribute::from(&attr) {
                None => {
                    warn!("Invalid attribute {} in script", attr);
                    return Ok(());
                }, Some(attr) => attr,
            };
            effect.bonuses.add_entry(Bonus::Attribute { attribute, amount });
            Ok(())
        });
        methods.add_method_mut("add_callback", |_, effect, cb: CallbackData| {
            effect.callbacks.push(cb);
            Ok(())
        });
        methods.add_method_mut("deactivate_with", |_, effect, ability: ScriptAbility| {
            effect.deactivate_with_ability = Some(ability.id);
            Ok(())
        });
    }
}

fn add_num_bonus(_lua: &Lua, effect: &mut ScriptEffect, args: (String, f32)) -> Result<()> {
    let (name, amount) = args;
    let name = name.to_lowercase();
    let amount_int = amount as i32;

    trace!("Adding numeric bonus {} to '{}'", amount, name);
    use sulis_rules::Bonus::*;
    match name.as_ref() {
        "armor" => effect.bonuses.add_entry(Armor(amount_int)),
        "ap" => effect.bonuses.add_entry(ActionPoints(amount_int)),
        "reach" => effect.bonuses.add_entry(Reach(amount)),
        "range" => effect.bonuses.add_entry(Range(amount)),
        "initiative" => effect.bonuses.add_entry(Initiative(amount_int)),
        "hit_points" => effect.bonuses.add_entry(HitPoints(amount_int)),
        "accuracy" => effect.bonuses.add_entry(Accuracy(amount_int)),
        "defense" => effect.bonuses.add_entry(Defense(amount_int)),
        "fortitude" => effect.bonuses.add_entry(Fortitude(amount_int)),
        "reflex" => effect.bonuses.add_entry(Reflex(amount_int)),
        "will" => effect.bonuses.add_entry(Will(amount_int)),
        "concealment" => effect.bonuses.add_entry(Concealment(amount_int)),
        "crit_threshold" => effect.bonuses.add_entry(CritThreshold(amount_int)),
        "hit_threshold" => effect.bonuses.add_entry(HitThreshold(amount_int)),
        "graze_threshold" => effect.bonuses.add_entry(GrazeThreshold(amount_int)),
        "graze_multiplier" => effect.bonuses.add_entry(GrazeMultiplier(amount)),
        "hit_multiplier" => effect.bonuses.add_entry(HitMultiplier(amount)),
        "crit_multiplier" => effect.bonuses.add_entry(CritMultiplier(amount)),
        "movement_rate" => effect.bonuses.add_entry(MovementRate(amount)),
        "attack_cost" => effect.bonuses.add_entry(AttackCost(amount_int)),
        _ => warn!("Attempted to add num bonus with invalid type '{}'", name),
    }
    Ok(())
}

const TURNS_TO_MILLIS: u32 = 5000;

fn apply(effect_data: &ScriptEffect, pgen: Option<ScriptParticleGenerator>,
         anim: Option<ScriptColorAnimation>) -> Result<()> {
    let area_state = GameState::area_state();
    let mut area_state = area_state.borrow_mut();
    let entity = area_state.get_entity(effect_data.parent);
    let duration = effect_data.duration * TURNS_TO_MILLIS;

    info!("Apply effect to '{}' with duration {}", entity.borrow().actor.actor.name, duration);
    let mut effect = Effect::new(&effect_data.name, duration, effect_data.bonuses.clone(),
        effect_data.deactivate_with_ability.clone());
    for cb in effect_data.callbacks.iter() {
        effect.add_callback(Rc::new(cb.clone()));
    }

    if let Some(ref pgen) = pgen {
        let pgen = script_particle_generator::create_pgen(&pgen, &area_state)?;
        pgen.add_removal_listener(&mut effect);
        GameState::add_animation(Box::new(pgen));
    }
    if let Some(ref anim) = anim {
        let anim = script_color_animation::create_anim(&anim, &area_state)?;
        anim.add_removal_listener(&mut effect);
        GameState::add_animation(Box::new(anim));
    }

    area_state.add_effect(&entity, effect);
    Ok(())
}
