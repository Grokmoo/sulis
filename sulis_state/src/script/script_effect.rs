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

use rlua::{Context, UserData, UserDataMethods};

use sulis_core::util::{ExtInt, Point};
use sulis_rules::{Attribute, Bonus, BonusKind, BonusList, Damage, DamageKind, bonus::{self, Contingent},
    WeaponKind, ArmorKind, Slot, WeaponStyle, ROUND_TIME_MILLIS};

use crate::script::{CallbackData, Result, script_particle_generator, ScriptParticleGenerator,
    script_color_animation, ScriptColorAnimation, ScriptAbility,
    script_scale_animation, ScriptScaleAnimation,
    script_image_layer_animation, ScriptImageLayerAnimation, ScriptCallback};
use crate::{effect, Effect, GameState};

/// Represents a surface that already exists, and is being passed into
/// a Lua script.  Not used during effect creation
/// # `mark_for_removal()`
/// Causes the referenced surface to be removed on the next frame.  This
/// is an asynchronous function.
#[derive(Clone, Debug)]
pub struct ScriptActiveSurface {
    pub index: usize,
}

impl UserData for ScriptActiveSurface {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("mark_for_removal", |_, surface, _args: ()| {
            let mgr = GameState::turn_manager();
            let mut mgr = mgr.borrow_mut();
            let effect = match mgr.effect_mut_checked(surface.index) {
                None => {
                    warn!("Effect index associated with ScriptSurface is invalid");
                    return Ok(());
                }, Some(effect) => effect,
            };
            effect.mark_for_removal();
            Ok(())
        });
    }
}

#[derive(Clone)]
enum Kind {
    Entity(usize),

    Surface {
        points: Vec<(i32, i32)>,
        squares_to_fire_on_moved: u32
    }
}

/// A user menu selection
///
/// # `value() -> String`
/// Returns the text value that the user selected
#[derive(Clone)]
pub struct ScriptMenuSelection {
    pub value: String,
}

impl UserData for ScriptMenuSelection {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("value", |_, selection, ()| {
            Ok(selection.value.to_string())
        });
    }
}

/// An already applied effect, in contrast to an effect being created
/// via `ScriptEntity:create_effect`
///
/// # `name() -> String`
/// Returns the user defined name of this effect
///
/// # `tag() -> String`
/// Returns the user defined tag of this effect
///
/// # `cur_duration() -> Int`
/// Returns the number of rounds this effect has currently been active
///
/// # `total_duration() -> Int`
/// Returns the total number of rounds this effect will be active for, or
/// 0 for infinite duration (modal) effects
///
/// # `total_duration_is_infinite() -> Bool`
/// Returns true if this effect has infinite duration (manually removed or modal),
/// false otherwise
///
/// # `has_bonus_of_kind(kind: String) -> Bool`
/// Checks whether this effect has one of more bonuses of the given kind.  The kind
/// Bonus kinds include `armor`, `ap`, `reach`, `range`, `initiative`, `hit_points`,
/// `melee_accuracy`, `ranged_accuracy`, `spell_accuracy`, `defense`, `fortitude`,
/// `reflex`, `will`, `concealment`, `concealment_ignore`, `crit_chance`,
/// `hit_threshold`, `graze_threshold`, `graze_multiplier`, `hit_multiplier`,
/// `crit_multiplier`, `movement_rate`, `attack_cost`, `ability_ap_cost`,
/// `hidden`, `free_ability_group_use`, abilities_disabled`, `move_disabled`,
/// `attack_disabled`, `flanked_immunity`, `sneak_attack_immunity`, `crit_immunity`
///
/// # `mark_for_removal()`
/// Marks this effect to be removed on the next update.  This is done asynchronously,
/// so the effect will still be applied when this method returns.
#[derive(Clone)]
pub struct ScriptAppliedEffect {
    index: usize,
    name: String,
    tag: String,
    cur_duration: u32,
    total_duration: ExtInt,
}

impl ScriptAppliedEffect {
    pub fn new(effect: &Effect, index: usize) -> ScriptAppliedEffect {
        ScriptAppliedEffect {
            index,
            name: effect.name.to_string(),
            tag: effect.tag.to_string(),
            cur_duration: effect.cur_duration,
            total_duration: effect.total_duration,
        }
    }
}

fn check_for_bonus(effect: &ScriptAppliedEffect, kind: String) -> bool {
    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();
    let effect = match mgr.effect_checked(effect.index) {
        None => {
            error!("Invalid ScriptAppliedEffect {}", effect.name);
            return false;
        }, Some(effect) => effect,
    };

    use sulis_rules::bonus::BonusKind::*;
    let kind = match kind.as_ref() {
        "ability_ap_cost" => AbilityActionPointCost(0),
        "armor" => Armor(0),
        "ap" => ActionPoints(0),
        "reach" => Reach(0.0),
        "range" => Range(0.0),
        "initiative" => Initiative(0),
        "hit_points" => HitPoints(0),
        "melee_accuracy" => MeleeAccuracy(0),
        "ranged_accuracy" => RangedAccuracy(0),
        "spell_accuracy" => SpellAccuracy(0),
        "defense" => Defense(0),
        "fortitude" => Fortitude(0),
        "reflex" => Reflex(0),
        "will" => Will(0),
        "concealment" => Concealment(0),
        "concealment_ignore" => ConcealmentIgnore(0),
        "crit_chance" => CritChance(0),
        "hit_threshold" => HitThreshold(0),
        "graze_threshold" => GrazeThreshold(0),
        "graze_multiplier" => GrazeMultiplier(0.0),
        "hit_multiplier" => HitMultiplier(0.0),
        "crit_multiplier" => CritMultiplier(0.0),
        "movement_rate" => MovementRate(0.0),
        "attack_cost" => AttackCost(0),
        "hidden" => Hidden,
        "free_ability_group_use" => FreeAbilityGroupUse,
        "abilities_disabled" => AbilitiesDisabled,
        "move_disabled" => MoveDisabled,
        "attack_disabled" => AttackDisabled,
        "flanked_immunity" => FlankedImmunity,
        "sneak_attack_immunity" => SneakAttackImmunity,
        "crit_immunity" => CritImmunity,
        _ => {
            warn!("Attempted to add num bonus with invalid type '{}'", kind);
            return false;
        }
    };

    for bonus in effect.bonuses.iter() {
        let check_bonus = Bonus { when: bonus.when, kind: kind.clone() };

        if bonus::merge_if_dup(bonus, &check_bonus).is_some() {
            return true;
        }
    }

    true
}

impl UserData for ScriptAppliedEffect {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("name", |_, effect, ()| {
            Ok(effect.name.to_string())
        });

        methods.add_method("tag", |_, effect, ()| {
            Ok(effect.tag.to_string())
        });

        methods.add_method("cur_duration", |_, effect, ()| {
            Ok(effect.cur_duration)
        });

        methods.add_method("total_duration", |_, effect, ()| {
            Ok(match effect.total_duration {
                ExtInt::Infinity => 0,
                ExtInt::Int(val) => val,
            })
        });

        methods.add_method("total_duration_is_infinite", |_, effect, ()| {
            Ok(effect.total_duration.is_infinite())
        });

        methods.add_method("has_bonus_of_kind", |_, effect, kind: String| {
            Ok(check_for_bonus(effect, kind))
        });

        methods.add_method("mark_for_removal", |_, effect, ()| {
            let mgr = GameState::turn_manager();
            let mut mgr = mgr.borrow_mut();
            let effect = mgr.effect_mut(effect.index);
            effect.mark_for_removal();
            Ok(())
        });
    }
}

/// An effect, normally created via `ScriptEntity:create_effect`.
/// The effect is then configured and then `apply()` is called.
///
/// # `apply()`
/// Sets this effect to active on the parent entity.
///
/// # `set_icon(icon: String, text: String)`
/// Sets the specified icon and text as the icon data for this effect.  This icon
/// is displayed in various places in the UI.
///
/// # `set_squares_to_fire_on_moved(squares: Int)`
/// Only has an effect on surfaces.  Sets the number of squares that an entity
/// must move within a surface in order to trigger an `OnMovedInSurface` script
/// event.
///
/// # `add_image_layer_anim(anim: ScriptImageLayerAnimation)`
/// Adds the specified `anim` to this effect.  The anim will have `apply()` called
/// when this effect has `apply()` called.  It will be removed when this effect is
/// removed.
///
/// # `add_color_anim(anim: ScriptColorAnimation)`
/// Adds the specified `anim` to this effect.  The anim will have `apply()` called
/// when this effect has `apply()` called.  It will be removed when this effect is
/// removed.
///
/// # `add_scale_anim(anim: ScriptScaleAnimation)`
/// Adds the specified `anim` to this effect.  The anim will have `apply()` called when
/// this effect has `apply()` called.  It will be removed when this effect is
/// removed.
///
/// # `add_anim(anim: ScriptParticleGenerator)`
/// Adds the specified `anim` to this effect.  The anim will have `apply()` called
/// when this effect has `apply()` called.  It will be removed when this effect is
/// removed.
///
/// # `add_callback(callback: CallbackData)`
/// Adds the specified `callback` to fire for entity's with this effect.
///
/// # `deactivate_with(ability: ScriptAbility)`
/// Sets this effect to be removed whenever the specified `ability` is deactivated.
/// The ability must be a mode.
///
/// # `set_tag(tag: String)`
/// Sets a tag to identify this effect as being of a particular type to other scripts.
/// Most notably, this is used when calling `remove_effects_with_tag` on a `ScriptEntity`
///
/// # `add_num_bonus(kind: String, amount: Float, when: String (Optional))`
/// Adds a numeric bonus that is applied to the parent entity when this effect is active.
/// Positive values are bonuses, while negative values are penalties.  `when` is optional
/// and specifies a condition that must be met for the bonus to be active.  By default,
/// the bonus is always applied.  Valid values are `always`, `attack_when_hidden`,
/// `attack_when_flanking`, `weapon_equipped <WEAPON_KIND>`,
/// `armor_equipped <ARMOR_KIND> <INVENTORY_SLOT>`, `weapon_style <WEAPON_STYLE>`,
/// `attack_with_weapon <WEAPON_KIND>`, `attack_with_damage_kind <DAMAGE_KIND>`
///
/// Bonus kinds include `armor`, `ap`, `reach`, `range`, `initiative`, `hit_points`,
/// `melee_accuracy`, `ranged_accuracy`, `spell_accuracy`, `defense`, `fortitude`,
/// `reflex`, `will`, `concealment`, `concealment_ignore`, `crit_chance`,
/// `hit_threshold`, `graze_threshold`, `graze_multiplier`, `hit_multiplier`,
/// `crit_multiplier`, `movement_rate`, `attack_cost`, `ability_ap_cost`
///
/// # `add_damage(min: Float, max: Float, ap: Float (Optional), when: String (Optional))`
/// Adds a damage bonus of the specified amount (from `min` to `max` randomly, with `ap`
/// armor piercing).  See `add_num_bonus`
///
/// # `add_hidden(when: String (Optional))`
/// Adds the hidden status to this effect.  See `add_num_bonus`
///
/// # `add_free_ability_group_use(when: String(Optional))`
/// Abbs ability use not using up group uses per encounter/day to this effect.  See `add_num_bonus`
///
/// # `add_abilities_disabled(when: String (Optional))`
/// Adds ability-use disabled status to this effect.  See `add_num_bonus`
///
/// # `add_move_disabled(when: String (Optional))`
/// Adds the move disabled status to this effect. See `add_num_bonus`
///
/// # `add_attack_disabled(when: String (Optional))`
/// Adds the attack disabled status to this effect.  See `add_num_bonus`
///
/// # `add_flanked_immunity(when: String (Optional))`
/// Adds immunity to flanking to this effect.  See `add_num_bonus`
///
/// # `add_sneak_attack_immunity(when: String (Optional))`
/// Adds immunity to sneak attack to this effect.  See `add_num_bonus`
///
/// # `add_crit_immunity(when: String (Optional))`
/// Adds immunity to crits to this effect (all crits become hits).  See `add_num_bonus`
///
/// # `add_damage_of_kind(min: Float, max: Float, kind: String, ap: String (Optional),
/// when: String (Optional))`
/// Adds the specified amount (from `min` to `max` randomly, with `ap` armor piercing)
/// of damage of the specified `kind` to this effect.
/// See `add_num_bonus`
///
/// # `add_armor_of_kind(value: Float, kind: String, when: String (Optional))`
/// Adds an armor bonus of the specified `value` and `kind` to this effect.  See
/// `add_num_bonus`
///
/// # `add_resistance(value: Float, kind: String, when: String (Optional))`
/// Adds a percentage damage resistance of `value` against `kind` damage
/// as a bonus to this effect.  See `add_num_bonus`
///
/// # `add_attribute_bonus(attr: String, amount: Float, when: String (Optional))`
/// Adds an attribute bonus for `attr` of `amount` to this effect.  Valid attributes
/// are `Strength`, `Dexterity`, `Endurance`, `Perception`, `Intellect`, and `Wisdom`
#[derive(Clone)]
pub struct ScriptEffect {
    kind: Kind,
    name: String,
    tag: String,
    duration: ExtInt,
    deactivate_with_ability: Option<String>,
    pub bonuses: BonusList,
    icon: Option<effect::Icon>,
    callbacks: Vec<CallbackData>,
    pgens: Vec<ScriptParticleGenerator>,
    image_layer_anims: Vec<ScriptImageLayerAnimation>,
    color_anims: Vec<ScriptColorAnimation>,
    scale_anims: Vec<ScriptScaleAnimation>,
}

impl ScriptEffect {
    pub fn new_surface(points: Vec<(i32, i32)>, name: &str, duration: ExtInt) -> ScriptEffect {
        ScriptEffect {
            kind: Kind::Surface { points, squares_to_fire_on_moved: 1 },
            name: name.to_string(),
            tag: "default".to_string(),
            deactivate_with_ability: None,
            duration,
            icon: None,
            bonuses: BonusList::default(),
            callbacks: Vec::new(),
            pgens: Vec::new(),
            image_layer_anims: Vec::new(),
            color_anims: Vec::new(),
            scale_anims: Vec::new(),
        }
    }

    pub fn new_entity(parent: usize, name: &str, duration: ExtInt) -> ScriptEffect {
        ScriptEffect {
            kind: Kind::Entity(parent),
            name: name.to_string(),
            tag: "default".to_string(),
            deactivate_with_ability: None,
            duration,
            icon: None,
            bonuses: BonusList::default(),
            callbacks: Vec::new(),
            pgens: Vec::new(),
            image_layer_anims: Vec::new(),
            color_anims: Vec::new(),
            scale_anims: Vec::new(),
        }
    }
}

impl UserData for ScriptEffect {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("apply", |_, effect, _args: ()| {
            apply(effect)
        });
        methods.add_method_mut("set_icon", |_, effect, (icon, text): (String, String)| {
            effect.icon = Some(effect::Icon { icon, text });
            Ok(())
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
        methods.add_method_mut("add_image_layer_anim", |_, effect, anim: ScriptImageLayerAnimation| {
            effect.image_layer_anims.push(anim);
            Ok(())
        });
        methods.add_method_mut("add_scale_anim", |_, effect, anim: ScriptScaleAnimation| {
            effect.scale_anims.push(anim);
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
                               (f32, f32, Option<f32>, Option<String>)| {
            let min = min as u32;
            let max = max as u32;
            let ap = ap.unwrap_or(0.0) as u32;
            let kind = BonusKind::Damage(Damage { min, max, ap, kind: None });
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_hidden", |_, effect, when: Option<String>| {
            let kind = BonusKind::Hidden;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_free_ability_group_use", |_, effect, when: Option<String>| {
            let kind = BonusKind::FreeAbilityGroupUse;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_abilities_disabled", |_, effect, when: Option<String>| {
            let kind = BonusKind::AbilitiesDisabled;
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
        methods.add_method_mut("add_flanked_immunity", |_, effect, when: Option<String>| {
            let kind = BonusKind::FlankedImmunity;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_sneak_attack_immunity", |_, effect, when: Option<String>| {
            let kind = BonusKind::SneakAttackImmunity;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_crit_immunity", |_, effect, when: Option<String>| {
            let kind = BonusKind::CritImmunity;
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_damage_of_kind", |_, effect, (min, max, kind, ap, when):
                               (f32, f32, String, Option<f32>, Option<String>)| {
            let min = min as u32;
            let max = max as u32;
            let ap = ap.unwrap_or(0.0) as u32;
            let dmg_kind = DamageKind::from_str(&kind);
            let kind = BonusKind::Damage(
                Damage { min, max, ap: ap, kind: Some(dmg_kind) }
            );
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_armor_of_kind", |_, effect, (value, kind, when):
                               (f32, String, Option<String>)| {
            let value = value as i32;
            let armor_kind = DamageKind::from_str(&kind);
            let kind = BonusKind::ArmorKind { kind: armor_kind, amount: value };
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_resistance", |_, effect, (value, kind, when):
                               (f32, String, Option<String>)| {
            let value = value as i32;
            let dmg_kind = DamageKind::from_str(&kind);
            let kind = BonusKind::Resistance { kind: dmg_kind, amount: value };
            add_bonus_to_effect(effect, kind, when);
            Ok(())
        });
        methods.add_method_mut("add_attribute_bonus", |_, effect, (attr, amount, when):
                               (String, f32, Option<String>)| {
            let amount = amount as i8;
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
                "threatened" => Contingent::Threatened,
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

fn add_num_bonus(_lua: Context, effect: &mut ScriptEffect, (name, amount, when):
                 (String, f32, Option<String>)) -> Result<()> {
    let name = name.to_lowercase();
    let amount_int = amount as i32;

    trace!("Adding numeric bonus {} to '{}'", amount, name);
    use sulis_rules::bonus::BonusKind::*;
    let kind = match name.as_ref() {
        "ability_ap_cost" => AbilityActionPointCost(amount_int),
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
        "crit_chance" => CritChance(amount_int),
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

fn apply(effect_data: &ScriptEffect) -> Result<()> {
    let mgr = GameState::turn_manager();
    let duration = effect_data.duration * ROUND_TIME_MILLIS;

    debug!("Apply effect with {}, {}, {}", effect_data.name, effect_data.tag, duration);
    let mut effect = Effect::new(&effect_data.name, &effect_data.tag, duration, effect_data.bonuses.clone(),
        effect_data.deactivate_with_ability.clone());
    if let Some(icon) = &effect_data.icon {
        effect.set_icon(icon.icon.clone(), icon.text.clone());
    }
    let cbs = effect_data.callbacks.clone();

    let effect_index = mgr.borrow().get_next_effect_index();

    let mut anims = Vec::new();
    let mut marked = Vec::new();
    for anim in effect_data.color_anims.iter() {
        anims.push(script_color_animation::create_anim(&anim)?);
    }

    for anim in effect_data.scale_anims.iter() {
        anims.push(script_scale_animation::create_anim(&anim)?);
    }

    for anim in effect_data.image_layer_anims.iter() {
        anims.push(script_image_layer_animation::create_anim(&anim)?);
    }

    for mut anim in anims {
        anim.set_removal_effect(effect_index);
        marked.push(anim.get_marked_for_removal());
        GameState::add_animation(anim);
    }

    match &effect_data.kind {
        Kind::Entity(parent) => {
            for pgen in effect_data.pgens.iter() {
                let mut pgen = script_particle_generator::create_pgen(&pgen, pgen.owned_model())?;
                pgen.set_removal_effect(effect_index);
                marked.push(pgen.get_marked_for_removal());
                GameState::add_animation(pgen);
            }

            let entity = mgr.borrow().entity(*parent);
            effect.set_owning_entity(entity.borrow().index());
            info!("Apply effect to '{}' with duration {}", entity.borrow().actor.actor.name, duration);
            // get the list of cbs before applying so it doesn't include itself
            let on_applied_cbs = entity.borrow().callbacks(&mgr.borrow());

            // apply the effect
            let index = mgr.borrow_mut().add_effect(effect, &entity, cbs, marked);

            // fire the on_applied cbs
            let sae = ScriptAppliedEffect::new(&mgr.borrow().effect(index), index);
            on_applied_cbs.iter().for_each(|cb| cb.on_effect_applied(sae.clone()));
        },
        Kind::Surface { points, squares_to_fire_on_moved } => {
            let points: Vec<_> = points.iter().map(|(x, y)| Point::new(*x, *y)).collect();
            for pgen in effect_data.pgens.iter() {
                for p in points.iter() {
                    let mut pgen = script_particle_generator::create_surface_pgen(&pgen, p.x, p.y)?;
                    pgen.set_removal_effect(effect_index);
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
