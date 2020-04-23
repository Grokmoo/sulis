//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::fmt::Display;

use sulis_core::ui::WidgetState;
use sulis_module::bonus::{AttackBuilder, AttackKindBuilder, Contingent};
use sulis_module::{Armor, Bonus, BonusList, DamageKind, Module, PrereqList};

pub fn format_bonus_or_penalty(amount: i32) -> String {
    if amount >= 0 {
        format!("+{}", amount)
    } else {
        format!("{}", amount)
    }
}

pub fn add_attack_text_args(attack: &AttackBuilder, widget_state: &mut WidgetState) {
    widget_state.add_text_arg("min_damage", &attack.damage.min.to_string());
    widget_state.add_text_arg("max_damage", &attack.damage.max.to_string());
    if attack.damage.ap > 0 {
        widget_state.add_text_arg("armor_penetration", &attack.damage.ap.to_string());
    }
    add_if_present(widget_state, "damage_kind", attack.damage.kind);

    match attack.kind {
        AttackKindBuilder::Melee { reach } => {
            widget_state.add_text_arg("reach", &reach.to_string())
        }
        AttackKindBuilder::Ranged { range, .. } => {
            widget_state.add_text_arg("range", &range.to_string())
        }
    }

    let bonuses = &attack.bonuses;
    add_if_nonzero(
        widget_state,
        "attack_crit_chance",
        bonuses.crit_chance as f32,
    );
    add_if_nonzero(
        widget_state,
        "attack_hit_threshold",
        bonuses.hit_threshold as f32,
    );
    add_if_nonzero(
        widget_state,
        "attack_graze_threshold",
        bonuses.graze_threshold as f32,
    );
    add_if_nonzero(
        widget_state,
        "attack_graze_multiplier",
        bonuses.graze_multiplier,
    );
    add_if_nonzero(
        widget_state,
        "attack_hit_multiplier",
        bonuses.hit_multiplier,
    );
    add_if_nonzero(
        widget_state,
        "attack_crit_multiplier",
        bonuses.crit_multiplier,
    );
    add_if_nonzero(
        widget_state,
        "attack_melee_accuracy",
        bonuses.melee_accuracy as f32,
    );
    add_if_nonzero(
        widget_state,
        "attack_ranged_accuracy",
        bonuses.ranged_accuracy as f32,
    );
    add_if_nonzero(
        widget_state,
        "attack_spell_accuracy",
        bonuses.spell_accuracy as f32,
    );

    if let Some(damage) = bonuses.damage {
        widget_state.add_text_arg("attack_min_bonus_damage", &damage.min.to_string());
        widget_state.add_text_arg("attack_max_bonus_damage", &damage.max.to_string());
        if let Some(kind) = damage.kind {
            widget_state.add_text_arg("attack_bonus_damage_kind", &kind.to_string());
        }
    }
}

fn add<T: Display>(widget_state: &mut WidgetState, name: &str, value: T) {
    widget_state.add_text_arg(name, &value.to_string());
}

fn find_index(group: &str, uses_so_far: &mut Vec<String>) -> usize {
    for (index, so_far_group) in uses_so_far.iter().enumerate() {
        if so_far_group == group {
            return index;
        }
    }

    uses_so_far.push(group.to_string());
    uses_so_far.len() - 1
}

fn add_bonus(
    bonus: &Bonus,
    state: &mut WidgetState,
    has_accuracy: &mut bool,
    group_uses_so_far: &mut Vec<String>,
    damage_index: &mut usize,
    armor: &mut Armor,
) {
    use sulis_module::BonusKind::*;
    match &bonus.kind {
        Attribute { attribute, amount } => add(state, &attribute.short_name(), amount),
        ActionPoints(amount) => add(state, "action_points", Module::rules().format_ap(*amount)),
        Armor(amount) => armor.add_base(*amount),
        ArmorKind { kind, amount } => armor.add_kind(*kind, *amount),
        Resistance { kind, amount } => add(
            state,
            &format!("resistance_{}", kind).to_lowercase(),
            *amount,
        ),
        Damage(damage) => {
            let index = *damage_index;
            if damage.max > 0 {
                add(state, &format!("min_bonus_damage_{}", index), damage.min);
                add(state, &format!("max_bonus_damage_{}", index), damage.max);
            }
            if damage.ap > 0 {
                add(state, &format!("armor_penetration_{}", index), damage.ap);
            }
            if let Some(kind) = damage.kind {
                add(state, &format!("bonus_damage_kind_{}", index), kind);
            }
            *damage_index += 1;
        }
        Reach(amount) => add(state, "bonus_reach", amount),
        Range(amount) => add(state, "bonus_range", amount),
        Initiative(amount) => add(state, "initiative", amount),
        HitPoints(amount) => add(state, "hit_points", amount),
        MeleeAccuracy(amount) => {
            add(state, "melee_accuracy", amount);
            *has_accuracy = true;
        }
        RangedAccuracy(amount) => {
            add(state, "ranged_accuracy", amount);
            *has_accuracy = true;
        }
        SpellAccuracy(amount) => {
            add(state, "spell_accuracy", amount);
            *has_accuracy = true;
        }
        Defense(amount) => add(state, "defense", amount),
        Fortitude(amount) => add(state, "fortitude", amount),
        Reflex(amount) => add(state, "reflex", amount),
        Will(amount) => add(state, "will", amount),
        Concealment(amount) => add(state, "concealment", amount),
        ConcealmentIgnore(amount) => add(state, "concealment_ignore", amount),
        CritChance(amount) => add(state, "crit_chance", amount),
        HitThreshold(amount) => add(state, "hit_threshold", amount),
        GrazeThreshold(amount) => add(state, "graze_threshold", amount),
        CritMultiplier(amount) => state.add_text_arg("crit_multiplier", &format!("{:.2}", amount)),
        HitMultiplier(amount) => state.add_text_arg("hit_multiplier", &format!("{:.2}", amount)),
        GrazeMultiplier(amount) => {
            state.add_text_arg("graze_multiplier", &format!("{:.2}", amount))
        }
        MovementRate(amount) => state.add_text_arg("movement_rate", &format!("{:.2}", amount)),
        MoveAnimRate(amount) => state.add_text_arg("move_anim_rate", &format!("{:.2}", amount)),
        CasterLevel(amount) => add(state, "caster_level", amount),
        AttackCost(amount) => {
            let cost = Module::rules().to_display_ap(*amount);
            add(state, "attack_cost", cost);
        }
        AbilityActionPointCost(amount) => {
            let cost = Module::rules().to_display_ap(*amount);
            add(state, "ability_ap_cost", cost);
        }
        GroupUsesPerEncounter { group, amount } => {
            let index = find_index(group, group_uses_so_far);
            add(state, &format!("ability_group_{}", index), group);
            add(
                state,
                &format!("ability_group_{}_uses_per_encounter", index),
                amount,
            );
        }
        GroupUsesPerDay { group, amount } => {
            let index = find_index(group, group_uses_so_far);
            add(state, &format!("ability_group_{}", index), group);
            add(
                state,
                &format!("ability_group_{}_uses_per_day", index),
                amount,
            );
        }
        ClassStat { id, amount } => {
            add(state, "class_stat_id", id);
            add(state, "class_stat_amount", amount);
        }
        ArmorProficiency(armor_kind) => {
            add(
                state,
                &format!("armor_proficiency_{:?}", armor_kind),
                "true",
            );
        }
        WeaponProficiency(weapon_kind) => {
            add(
                state,
                &format!("weapon_proficiency_{:?}", weapon_kind),
                "true",
            );
        }
        FlankingAngle(amount) => add(state, "flanking_angle", amount),
        FreeAbilityGroupUse => add(state, "free_ability_group_use", true),
        AbilitiesDisabled => add(state, "abilities_disabled", true),
        MoveDisabled => add(state, "move_disabled", true),
        AttackDisabled => add(state, "attack_disabled", true),
        Hidden => add(state, "hidden", true),
        FlankedImmunity => add(state, "flanked_immunity", true),
        SneakAttackImmunity => add(state, "sneak_attack_immunity", true),
        CritImmunity => add(state, "crit_immunity", true),
    }
}

pub fn add_prereq_text_args(prereqs: &PrereqList, state: &mut WidgetState) {
    state.add_text_arg("prereqs", "true");

    if let Some(ref attrs) = prereqs.attributes {
        for &(attr, amount) in attrs.iter() {
            state.add_text_arg(
                &format!("prereq_{}", attr.short_name()),
                &amount.to_string(),
            );
        }
    }

    for (index, &(ref class_id, level)) in prereqs.levels.iter().enumerate() {
        let class = match Module::class(class_id) {
            None => {
                warn!("Invalid class '{}' in prereq list", class_id);
                continue;
            }
            Some(class) => class,
        };
        state.add_text_arg(&format!("prereq_class_{}", index), &class.name);
        state.add_text_arg(&format!("prereq_level_{}", index), &level.to_string());
    }

    if let Some(total_level) = prereqs.total_level {
        state.add_text_arg("prereq_total_level", &total_level.to_string());
    }

    if let Some(ref race_id) = prereqs.race {
        match Module::race(race_id) {
            None => {
                warn!("Invalid race '{}' in prereq list", race_id);
            }
            Some(race) => {
                state.add_text_arg("prereq_race", &race.name);
            }
        }
    }

    for (index, ref ability_id) in prereqs.abilities.iter().enumerate() {
        let ability = match Module::ability(ability_id) {
            None => {
                warn!("No ability '{}' found for prereq list", ability_id);
                continue;
            }
            Some(ability) => ability,
        };

        state.add_text_arg(&format!("prereq_ability_{}", index), &ability.name);
    }
}

pub fn add_bonus_text_args(bonuses: &BonusList, widget_state: &mut WidgetState) {
    let mut group_uses_so_far = Vec::new();
    let mut damage_index = 0;
    let mut armor = Armor::default();
    let mut has_accuracy = false;
    for bonus in bonuses.iter() {
        match bonus.when {
            Contingent::Always => (),
            _ => continue,
        }
        add_bonus(
            bonus,
            widget_state,
            &mut has_accuracy,
            &mut group_uses_so_far,
            &mut damage_index,
            &mut armor,
        );
    }

    if has_accuracy {
        add(widget_state, "any_accuracy", "true");
    }

    if !armor.is_empty() {
        add(widget_state, "any_armor", "true");
    }
    if armor.base() > 0 {
        add(widget_state, "armor", armor.base());
    }

    for kind in DamageKind::iter() {
        if !armor.differs_from_base(*kind) {
            continue;
        }
        add(
            widget_state,
            &format!("armor_{}", kind).to_lowercase(),
            armor.amount(*kind),
        );
    }
}

fn add_if_nonzero(widget_state: &mut WidgetState, text: &str, val: f32) {
    if val != 0.0 {
        widget_state.add_text_arg(text, &val.to_string());
    }
}

fn add_if_present<T: Display>(widget_state: &mut WidgetState, text: &str, val: Option<T>) {
    if let Some(val) = val {
        widget_state.add_text_arg(text, &val.to_string());
    }
}
