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

use std::str::FromStr;

use rlua::{Lua, UserData, UserDataMethods};

use sulis_core::util::{ExtInt, Point};
use sulis_rules::{Attribute, Bonus, BonusKind, BonusList, Damage, DamageKind, bonus::Contingent,
    WeaponKind, ArmorKind, Slot, WeaponStyle};

use script::{CallbackData, Result, script_particle_generator, ScriptParticleGenerator,
    script_color_animation, ScriptColorAnimation, ScriptAbility};
use {Effect, GameState};

#[derive(Clone)]
enum Kind {
    Entity(usize),

    Surface {
        points: Vec<(i32, i32)>,
        squares_to_fire_on_moved: u32
    }
}

#[derive(Clone)]
pub struct ScriptEffect {
    kind: Kind,
    name: String,
    tag: String,
    duration: ExtInt,
    deactivate_with_ability: Option<String>,
    pub bonuses: BonusList,
    callbacks: Vec<CallbackData>,
    pgens: Vec<ScriptParticleGenerator>,
    color_anims: Vec<ScriptColorAnimation>,
}

impl ScriptEffect {
    pub fn new_surface(points: Vec<(i32, i32)>, name: &str, duration: ExtInt) -> ScriptEffect {
        ScriptEffect {
            kind: Kind::Surface { points, squares_to_fire_on_moved: 1 },
            name: name.to_string(),
            tag: "default".to_string(),
            deactivate_with_ability: None,
            duration,
            bonuses: BonusList::default(),
            callbacks: Vec::new(),
            pgens: Vec::new(),
            color_anims: Vec::new(),
        }
    }

    pub fn new_entity(parent: usize, name: &str, duration: ExtInt) -> ScriptEffect {
        ScriptEffect {
            kind: Kind::Entity(parent),
            name: name.to_string(),
            tag: "default".to_string(),
            deactivate_with_ability: None,
            duration,
            bonuses: BonusList::default(),
            callbacks: Vec::new(),
            pgens: Vec::new(),
            color_anims: Vec::new(),
        }
    }
}

impl UserData for ScriptEffect {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("apply", |_, effect, _args: ()| {
            apply(effect)
        });
        methods.add_method_mut("set_squares_to_fire_on_moved", |_, effect, squares: u32| {
            match effect.kind {
                Kind::Entity(_) => {
                    warn!("Attempted to set movement squares until on_moved fired for non surface effect");
                },
                Kind::Surface { ref mut squares_to_fire_on_moved, .. } => {
                    *squares_to_fire_on_moved = squares;
                }
            }
            Ok(())
        });
        methods.add_method_mut("add_color_anim", |_, effect, anim: ScriptColorAnimation| {
            effect.color_anims.push(anim);
            Ok(())
        });
        methods.add_method_mut("add_anim", |_, effect, pgen: ScriptParticleGenerator| {
            effect.pgens.push(pgen);
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
        methods.add_method_mut("set_tag", |_, effect, tag: String| {
            effect.tag = tag;
            Ok(())
        });
        methods.add_method_mut("add_num_bonus", &add_num_bonus);
        methods.add_method_mut("add_damage", |_, effect, (min, max, ap, when):
                               (u32, u32, Option<u32>, Option<String>)| {
            let kind = BonusKind::Damage(Damage { min, max, ap: ap.unwrap_or(0), kind: None });
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_hidden", |_, effect, when: Option<String>| {
            let kind = BonusKind::Hidden;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_move_disabled", |_, effect, when: Option<String>| {
            let kind = BonusKind::MoveDisabled;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_attack_disabled", |_, effect, when: Option<String>| {
            let kind = BonusKind::AttackDisabled;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_damage_of_kind", |_, effect, (min, max, kind, ap, when):
                               (u32, u32, String, Option<u32>, Option<String>)| {
            let dmg_kind = DamageKind::from_str(&kind);
            let kind = BonusKind::Damage(
                Damage { min, max, ap: ap.unwrap_or(0), kind: Some(dmg_kind) }
            );
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_armor_of_kind", |_, effect, (value, kind, when):
                               (i32, String, Option<String>)| {
            let armor_kind = DamageKind::from_str(&kind);
            let kind = BonusKind::ArmorKind { kind: armor_kind, amount: value };
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_attribute_bonus", |_, effect, (attr, amount, when):
                               (String, i8, Option<String>)| {
            let attribute = match Attribute::from(&attr) {
                None => {
                    warn!("Invalid attribute {} in script", attr);
                    return Ok(());
                }, Some(attr) => attr,
            };
            let kind = BonusKind::Attribute { attribute, amount };
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
    }
}

fn add_bonus_to_effect(effect: &mut ScriptEffect, bonus_kind: BonusKind, when: Option<String>) {
    if let Some(when) = when {
        let split: Vec<_> = when.split(" ").collect();
        if split.is_empty() {
            warn!("Unable to parse bonus when of '{}'", when);
            return;
        }

        let contingent = if split.len() == 1 {
            match split[0] {
                "always" => Contingent::Always,
                "attack_when_hidden" => Contingent::AttackWhenHidden,
                "attack_when_flanking" => Contingent::AttackWhenFlanking,
                _ => {
                    warn!("Unable to parse contingent '{}'.  May need an additional arg.", when);
                    return;
                }
            }
        } else {
            match split[0] {
                "weapon_equipped" => {
                    if split.len() != 2 {
                        warn!("Need 2 args for weapon_equipped from '{}'", when);
                        return;
                    }

                    if let Ok(weapon_kind) = WeaponKind::from_str(split[1]) {
                        Contingent::WeaponEquipped(weapon_kind)
                    } else {
                        return;
                    }
                },
                "armor_equipped" => {
                    if split.len() != 3 {
                        warn!("Need 3 args for armor_equipped from '{}'", when);
                        return;
                    }

                    let armor_kind = match ArmorKind::from_str(split[1]) {
                        Err(_) => return,
                        Ok(kind) => kind,
                    };

                    let slot = match Slot::from_str(split[2]) {
                        Err(_) => return,
                        Ok(slot) => slot,
                    };

                    Contingent::ArmorEquipped { kind: armor_kind, slot }
                },
                "weapon_style" => {
                    if split.len() != 2 {
                        warn!("Need 2 args for weapon_style from '{}'", when);
                        return;
                    }

                    if let Ok(weapon_style) = WeaponStyle::from_str(split[1]) {
                        Contingent::WeaponStyle(weapon_style)
                    } else {
                        return;
                    }
                },
                "attack_with_weapon" => {
                    if split.len() != 2 {
                        warn!("Need 2 args for attack_with_weapon from '{}'", when);
                        return;
                    }

                    if let Ok(weapon_kind) = WeaponKind::from_str(split[1]) {
                        Contingent::AttackWithWeapon(weapon_kind)
                    } else {
                        return;
                    }
                },
                "attack_with_damage_kind" => {
                    if split.len() != 2 {
                        warn!("Need 2 args for attack_with_damage_kind from '{}'", when);
                        return;
                    }

                    Contingent::AttackWithDamageKind(DamageKind::from_str(split[1]))
                },
                _ => {
                    warn!("Unable to parse contingent '{}'.  Unknown kind / too many args.", when);
                    return
                }
            }
        };

        let bonus = Bonus { when: contingent, kind: bonus_kind };
        effect.bonuses.add(bonus);
    } else {
        effect.bonuses.add_kind(bonus_kind);
    }
}

fn add_num_bonus(_lua: &Lua, effect: &mut ScriptEffect, (name, amount, when):
                 (String, f32, Option<String>)) -> Result<()> {
    let name = name.to_lowercase();
    let amount_int = amount as i32;

    trace!("Adding numeric bonus {} to '{}'", amount, name);
    use sulis_rules::bonus::BonusKind::*;
    let kind = match name.as_ref() {
        "armor" => Armor(amount_int),
        "ap" => ActionPoints(amount_int),
        "reach" => Reach(amount),
        "range" => Range(amount),
        "initiative" => Initiative(amount_int),
        "hit_points" => HitPoints(amount_int),
        "melee_accuracy" => MeleeAccuracy(amount_int),
        "ranged_accuracy" => RangedAccuracy(amount_int),
        "spell_accuracy" => SpellAccuracy(amount_int),
        "defense" => Defense(amount_int),
        "fortitude" => Fortitude(amount_int),
        "reflex" => Reflex(amount_int),
        "will" => Will(amount_int),
        "concealment" => Concealment(amount_int),
        "concealment_ignore" => ConcealmentIgnore(amount_int),
        "crit_threshold" => CritThreshold(amount_int),
        "hit_threshold" => HitThreshold(amount_int),
        "graze_threshold" => GrazeThreshold(amount_int),
        "graze_multiplier" => GrazeMultiplier(amount),
        "hit_multiplier" => HitMultiplier(amount),
        "crit_multiplier" => CritMultiplier(amount),
        "movement_rate" => MovementRate(amount),
        "attack_cost" => AttackCost(amount_int),
        _ => {
            warn!("Attempted to add num bonus with invalid type '{}'", name);
            return Ok(());
        }
    };

    add_bonus_to_effect(effect, kind, when);
    Ok(())
}

const TURNS_TO_MILLIS: u32 = 5000;

fn apply(effect_data: &ScriptEffect) -> Result<()> {
    let mgr = GameState::turn_manager();
    let duration = effect_data.duration * TURNS_TO_MILLIS;

    let mut effect = Effect::new(&effect_data.name, &effect_data.tag, duration, effect_data.bonuses.clone(),
        effect_data.deactivate_with_ability.clone());
    let cbs = effect_data.callbacks.clone();

    let mut marked = Vec::new();
    for anim in effect_data.color_anims.iter() {
        let anim = script_color_animation::create_anim(&anim)?;
        marked.push(anim.get_marked_for_removal());
        GameState::add_animation(anim);
    }

    match &effect_data.kind {
        Kind::Entity(parent) => {
            for pgen in effect_data.pgens.iter() {
                let pgen = script_particle_generator::create_pgen(&pgen, pgen.owned_model())?;
                marked.push(pgen.get_marked_for_removal());
                GameState::add_animation(pgen);
            }

            let entity = mgr.borrow().entity(*parent);
            info!("Apply effect to '{}' with duration {}", entity.borrow().actor.actor.name, duration);
            mgr.borrow_mut().add_effect(effect, &entity, cbs, marked);
        },
        Kind::Surface { points, squares_to_fire_on_moved } => {
            let points: Vec<_> = points.iter().map(|(x, y)| Point::new(*x, *y)).collect();
            for pgen in effect_data.pgens.iter() {
                for p in points.iter() {
                    let pgen = script_particle_generator::create_surface_pgen(&pgen, p.x, p.y)?;
                    marked.push(pgen.get_marked_for_removal());
                    GameState::add_animation(pgen);
                }
            }
            let area = GameState::area_state();
            effect.set_surface_for_area(&area.borrow().area.id, &points, *squares_to_fire_on_moved);
            info!("Add surface to '{}' with duration {}", area.borrow().area.name, duration);
            mgr.borrow_mut().add_surface(effect, &area, points, cbs, marked);
        },
    }

    Ok(())
}
