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

use std::collections::HashMap;
use std::rc::Rc;

use crate::rules::bonus::{AttackBonuses, AttackBuilder, Bonus, BonusKind, BonusList};
use crate::rules::{
    AccuracyKind, Armor, ArmorKind, Attack, AttributeList, Damage, HitKind, Resistance, Slot,
    WeaponKind, WeaponStyle,
};
use crate::{Actor, Module};
use sulis_core::image::Image;
use sulis_core::util::{gen_rand, ExtInt};

#[derive(Clone)]
pub struct StatList {
    attack_range: f32,
    touch_range: f32,

    pub attributes: AttributeList,
    armor_proficiencies: Vec<ArmorKind>,
    weapon_proficiencies: Vec<WeaponKind>,

    // contingent bonuses are accumulated here and then applied if applicable when finalizing
    pub contingent_bonuses: BonusList,
    // bonuses contingent on flanking that are only applied to some attacks
    pub flanking_bonuses: BonusList,

    // these bonuses are applied only to the attack itself of the given weaponkind
    pub attack_bonuses: Vec<Bonus>,

    pub bonus_ap: i32,
    pub bonus_ability_action_point_cost: i32,
    pub bonus_damage: Vec<Damage>,
    pub bonus_reach: f32,
    pub bonus_range: f32,
    pub attacks: Vec<Attack>,
    pub armor: Armor,
    pub resistance: Resistance,
    pub max_hp: i32,
    pub initiative: i32,
    pub flanking_angle: i32,
    pub melee_accuracy: i32,
    pub ranged_accuracy: i32,
    pub spell_accuracy: i32,
    pub defense: i32,
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
    pub concealment: i32,
    pub concealment_ignore: i32,
    pub crit_chance: i32,
    pub hit_threshold: i32,
    pub graze_threshold: i32,
    pub graze_multiplier: f32,
    pub hit_multiplier: f32,
    pub crit_multiplier: f32,
    pub movement_rate: f32,
    pub attack_cost: i32,
    pub move_disabled: bool,
    pub attack_disabled: bool,
    pub abilities_disabled: bool,
    pub hidden: bool,
    pub flanked_immunity: bool,
    pub sneak_attack_immunity: bool,
    pub crit_immunity: bool,
    pub free_ability_group_use: bool,
    pub caster_level: i32,
    group_uses_per_encounter: HashMap<String, ExtInt>,
    group_uses_per_day: HashMap<String, ExtInt>,
    class_stats: HashMap<String, ExtInt>,
}

impl StatList {
    pub fn new(attrs: AttributeList) -> StatList {
        StatList {
            attributes: attrs,
            armor_proficiencies: Vec::new(),
            weapon_proficiencies: Vec::new(),
            contingent_bonuses: BonusList::default(),
            flanking_bonuses: BonusList::default(),
            attack_bonuses: Vec::new(),
            bonus_ap: 0,
            bonus_ability_action_point_cost: 0,
            bonus_damage: Vec::new(),
            bonus_reach: 0.0,
            bonus_range: 0.0,
            attack_range: 0.0,
            touch_range: 0.0,
            attacks: Vec::new(),
            armor: Armor::default(),
            resistance: Resistance::default(),
            max_hp: 0,
            initiative: 0,
            flanking_angle: 0,
            melee_accuracy: 0,
            ranged_accuracy: 0,
            spell_accuracy: 0,
            defense: 0,
            fortitude: 0,
            reflex: 0,
            will: 0,
            concealment: 0,
            concealment_ignore: 0,
            crit_chance: 0,
            hit_threshold: 0,
            graze_threshold: 0,
            graze_multiplier: 0.0,
            hit_multiplier: 0.0,
            crit_multiplier: 0.0,
            movement_rate: 0.0,
            attack_cost: 0,
            move_disabled: false,
            attack_disabled: false,
            abilities_disabled: false,
            hidden: false,
            flanked_immunity: false,
            sneak_attack_immunity: false,
            crit_immunity: false,
            free_ability_group_use: false,
            caster_level: 0,
            group_uses_per_encounter: HashMap::new(),
            group_uses_per_day: HashMap::new(),
            class_stats: HashMap::new(),
        }
    }

    pub fn class_stat_max(&self, stat: &str) -> ExtInt {
        self.class_stats
            .get(stat)
            .copied()
            .unwrap_or(ExtInt::Int(0))
    }

    pub fn uses_per_day_iter(&self) -> impl Iterator<Item = (&String, &ExtInt)> {
        self.group_uses_per_day.iter()
    }

    pub fn uses_per_encounter_iter(&self) -> impl Iterator<Item = (&String, &ExtInt)> {
        self.group_uses_per_encounter.iter()
    }

    pub fn uses_per_day(&self, ability_group: &str) -> ExtInt {
        *self
            .group_uses_per_day
            .get(ability_group)
            .unwrap_or(&ExtInt::Int(0))
    }

    pub fn uses_per_encounter(&self, ability_group: &str) -> ExtInt {
        *self
            .group_uses_per_encounter
            .get(ability_group)
            .unwrap_or(&ExtInt::Int(0))
    }

    pub fn has_armor_proficiency(&self, prof: ArmorKind) -> bool {
        self.armor_proficiencies.contains(&prof)
    }

    pub fn has_weapon_proficiency(&self, prof: WeaponKind) -> bool {
        self.weapon_proficiencies.contains(&prof)
    }

    pub fn attack_roll(
        &self,
        accuracy_kind: AccuracyKind,
        crit_immunity: bool,
        defense: i32,
        bonuses: &AttackBonuses,
    ) -> HitKind {
        let accuracy = match accuracy_kind {
            AccuracyKind::Melee => self.melee_accuracy + bonuses.melee_accuracy,
            AccuracyKind::Ranged => self.ranged_accuracy + bonuses.ranged_accuracy,
            AccuracyKind::Spell => self.spell_accuracy + bonuses.spell_accuracy,
        };
        let roll = gen_rand(1, 101);
        debug!(
            "Attack roll: {} with accuracy {} against {}",
            roll, accuracy, defense
        );

        if roll + accuracy < defense {
            return HitKind::Miss;
        }

        let result = roll + accuracy - defense;

        if !crit_immunity && (100 - roll) < self.crit_chance + bonuses.crit_chance {
            let roll2 = gen_rand(1, 101);
            let result2 = roll2 + accuracy - defense;
            if result2 > self.graze_threshold + bonuses.graze_threshold {
                HitKind::Crit
            } else {
                HitKind::Hit
            }
        } else if result > self.hit_threshold + bonuses.hit_threshold {
            HitKind::Hit
        } else if result > self.graze_threshold + bonuses.graze_threshold {
            HitKind::Graze
        } else {
            HitKind::Miss
        }
    }

    pub fn attack_is_melee(&self) -> bool {
        if self.attacks.is_empty() {
            return false;
        }

        self.attacks[0].is_melee()
    }

    pub fn attack_is_ranged(&self) -> bool {
        if self.attacks.is_empty() {
            return false;
        }

        self.attacks[0].is_ranged()
    }

    pub fn get_ranged_projectile(&self) -> Option<Rc<dyn Image>> {
        if !self.attack_is_ranged() {
            return None;
        }

        self.attacks[0].get_ranged_projectile()
    }

    /// Returns the max distance that this StatList can touch / reach
    pub fn touch_distance(&self) -> f32 {
        self.touch_range
    }

    /// Returns the maximum distance that this StatList's
    /// attacks can reach
    pub fn attack_distance(&self) -> f32 {
        self.attack_range
    }

    pub fn add_single_group_uses_per_day(&mut self, group_id: &str, uses: ExtInt) {
        let cur_uses = *self
            .group_uses_per_day
            .get(group_id)
            .unwrap_or(&ExtInt::Int(0));
        let new_uses = cur_uses + uses;
        self.group_uses_per_day
            .insert(group_id.to_string(), new_uses);
    }

    pub fn add_single_group_uses_per_encounter(&mut self, group_id: &str, uses: ExtInt) {
        let cur_uses = *self
            .group_uses_per_encounter
            .get(group_id)
            .unwrap_or(&ExtInt::Int(0));
        let new_uses = cur_uses + uses;
        self.group_uses_per_encounter
            .insert(group_id.to_string(), new_uses);
    }

    pub fn add_single_class_stat_i32(&mut self, stat_id: &str, amount: i32) {
        let cur_amount = *self.class_stats.get(stat_id).unwrap_or(&ExtInt::Int(0));
        let new_amount = if amount >= 0 {
            cur_amount + (amount as u32)
        } else {
            cur_amount + (-amount as u32)
        };
        self.class_stats.insert(stat_id.to_string(), new_amount);
    }

    pub fn add_single_class_stat_max(&mut self, stat_id: String, amount: ExtInt) {
        let cur_amount = *self.class_stats.get(&stat_id).unwrap_or(&ExtInt::Int(0));
        let new_amount = cur_amount + amount;
        self.class_stats.insert(stat_id, new_amount);
    }

    /// Adds the bonuses from the specified BonusList to this stat list.
    pub fn add(&mut self, bonuses: &BonusList) {
        self.add_multiple(bonuses, 1);
    }

    /// Adds the specified bonuses to this StatList the specified number of times.
    /// Note that non-numeric bonuses are only added once regardless of the value of
    /// times
    pub fn add_multiple(&mut self, bonuses: &BonusList, times: u32) {
        if times == 0 {
            return;
        }

        // TODO handle add multiple for weapon and attack bonuses
        for bonus in bonuses.iter() {
            use crate::rules::bonus::Contingent::*;
            match bonus.when {
                Always => self.add_bonus(&bonus.kind, times),
                AttackWithWeapon(_) | AttackWhenHidden | AttackWithDamageKind(_) => {
                    self.attack_bonuses.push(bonus.clone())
                }
                AttackWhenFlanking => self.flanking_bonuses.add(bonus.clone()),
                WeaponEquipped(_) | ArmorEquipped { .. } | WeaponStyle(_) | Threatened => {
                    self.contingent_bonuses.add(bonus.clone())
                }
            }
        }
    }

    fn add_bonus(&mut self, bonus: &BonusKind, times: u32) {
        let times_i32 = times as i32;
        let times_f32 = times as f32;

        use crate::rules::bonus::BonusKind::*;
        match bonus {
            AbilityActionPointCost(amount) => {
                self.bonus_ability_action_point_cost += amount * times_i32
            }
            Attribute { attribute, amount } => self.attributes.add(*attribute, *amount),
            ActionPoints(amount) => self.bonus_ap += amount * times_i32,
            Armor(amount) => self.armor.add_base(amount * times_i32),
            ArmorKind { kind, amount } => self.armor.add_kind(*kind, amount * times_i32),
            Resistance { kind, amount } => self.resistance.add_kind(*kind, amount * times_i32),
            Damage(damage) => self.bonus_damage.push(damage.mult(times)),
            ArmorProficiency(kind) => {
                if !self.armor_proficiencies.contains(kind) {
                    self.armor_proficiencies.push(*kind);
                }
            }
            WeaponProficiency(kind) => {
                if !self.weapon_proficiencies.contains(kind) {
                    self.weapon_proficiencies.push(*kind);
                }
            }
            Reach(amount) => self.bonus_reach += amount * times_f32,
            Range(amount) => self.bonus_range += amount * times_f32,
            Initiative(amount) => self.initiative += amount * times_i32,
            HitPoints(amount) => self.max_hp += amount * times_i32,
            MeleeAccuracy(amount) => self.melee_accuracy += amount * times_i32,
            RangedAccuracy(amount) => self.ranged_accuracy += amount * times_i32,
            SpellAccuracy(amount) => self.spell_accuracy += amount * times_i32,
            Defense(amount) => self.defense += amount * times_i32,
            Fortitude(amount) => self.fortitude += amount * times_i32,
            Reflex(amount) => self.reflex += amount * times_i32,
            Will(amount) => self.will += amount * times_i32,
            Concealment(amount) => self.concealment += amount * times_i32,
            ConcealmentIgnore(amount) => self.concealment_ignore += amount * times_i32,
            CritChance(amount) => self.crit_chance += amount * times_i32,
            HitThreshold(amount) => self.hit_threshold -= amount * times_i32,
            GrazeThreshold(amount) => self.graze_threshold -= amount * times_i32,
            CritMultiplier(amount) => self.crit_multiplier += amount * times_f32,
            HitMultiplier(amount) => self.hit_multiplier += amount * times_f32,
            GrazeMultiplier(amount) => self.graze_multiplier += amount * times_f32,
            MovementRate(amount) => self.movement_rate += amount * times_f32,
            AttackCost(amount) => self.attack_cost -= amount * times_i32,
            FlankingAngle(amount) => self.flanking_angle -= amount * times_i32,
            CasterLevel(amount) => self.caster_level += amount * times_i32,
            FreeAbilityGroupUse => self.free_ability_group_use = true,
            AbilitiesDisabled => self.abilities_disabled = true,
            MoveDisabled => self.move_disabled = true,
            AttackDisabled => self.attack_disabled = true,
            Hidden => self.hidden = true,
            FlankedImmunity => self.flanked_immunity = true,
            SneakAttackImmunity => self.sneak_attack_immunity = true,
            CritImmunity => self.crit_immunity = true,
            GroupUsesPerEncounter { group, amount } => {
                self.add_single_group_uses_per_encounter(group, *amount)
            }
            GroupUsesPerDay { group, amount } => self.add_single_group_uses_per_day(group, *amount),
            ClassStat { id, amount } => self.add_single_class_stat_i32(id, *amount),
        }
    }

    pub fn finalize<'a>(
        &mut self,
        actor: &'a Actor,
        mut attacks: Vec<(&'a AttackBuilder, WeaponKind)>,
        equipped_armor: HashMap<Slot, ArmorKind>,
        weapon_style: WeaponStyle,
        threatened: bool,
    ) {
        let rules = Module::rules();

        // clone here to avoid problem with add_bonus needing mutable borrow,
        // even though it would be safe
        let contingent = self.contingent_bonuses.clone();
        for bonus in contingent.iter() {
            use crate::rules::bonus::Contingent::*;
            match bonus.when {
                Always
                | AttackWithWeapon(_)
                | AttackWhenHidden
                | AttackWhenFlanking
                | AttackWithDamageKind(_) => unreachable!(),
                WeaponEquipped(weapon_kind) => {
                    for (_, attack_weapon_kind) in attacks.iter() {
                        if weapon_kind == *attack_weapon_kind {
                            self.add_bonus(&bonus.kind, 1);
                        }
                    }
                }
                ArmorEquipped { kind, slot } => {
                    if let Some(armor_kind) = equipped_armor.get(&slot) {
                        if *armor_kind == kind {
                            self.add_bonus(&bonus.kind, 1);
                        }
                    }
                }
                WeaponStyle(style) => {
                    if weapon_style == style {
                        self.add_bonus(&bonus.kind, 1);
                    }
                }
                Threatened => {
                    if threatened {
                        self.add_bonus(&bonus.kind, 1);
                    }
                }
            }
        }

        let multiplier = if attacks.is_empty() {
            attacks.push((&actor.race.base_attack, WeaponKind::Simple));
            1.0
        } else if attacks.len() > 1 {
            rules.dual_wield_damage_multiplier
        } else {
            1.0
        };
        let is_melee = attacks[0].0.is_melee();

        let mut attack_range = None;
        for (builder, weapon_kind) in attacks {
            let attack = Attack::new(builder, &self, weapon_kind).mult(multiplier);

            if attack_range.is_none() {
                attack_range = Some(attack.distance());
            } else {
                let cur_range = attack_range.unwrap();
                if attack.distance() < cur_range {
                    attack_range = Some(attack.distance());
                }
            }

            self.attacks.push(attack);
        }

        self.attack_range = attack_range.unwrap_or(0.0);

        let base_accuracy = rules.base_accuracy as i32;
        let base_defense = rules.base_defense as i32;
        let base_attr = rules.base_attribute;

        use crate::rules::Attribute::*;
        let attrs = &self.attributes;
        let str_bonus = attrs.bonus(Strength, base_attr);
        let dex_bonus = attrs.bonus(Dexterity, base_attr);
        let end_bonus = attrs.bonus(Endurance, base_attr);
        let per_bonus = attrs.bonus(Perception, base_attr);
        let int_bonus = attrs.bonus(Intellect, base_attr);
        let wis_bonus = attrs.bonus(Wisdom, base_attr);
        self.initiative += dex_bonus / 2 + per_bonus / 2;
        self.melee_accuracy += base_accuracy + per_bonus + str_bonus * 2;
        self.ranged_accuracy += base_accuracy + per_bonus + dex_bonus * 2;
        self.spell_accuracy += base_accuracy + wis_bonus + int_bonus * 2;
        self.defense += base_defense + dex_bonus * 2;
        self.fortitude += base_defense + end_bonus * 2;
        self.reflex += base_defense + dex_bonus * 2;
        self.will += base_defense + wis_bonus * 2;
        self.max_hp += (actor.total_level as i32 * end_bonus) / 3;

        let damage_stat_bonus = if is_melee { str_bonus } else { dex_bonus } as f32;

        self.graze_multiplier += 0.02 * damage_stat_bonus;
        self.hit_multiplier += 0.04 * damage_stat_bonus;
        self.crit_multiplier += 0.08 * damage_stat_bonus;

        if self.hit_multiplier < self.graze_multiplier {
            self.hit_multiplier = self.graze_multiplier;
        }

        if self.crit_multiplier < self.hit_multiplier {
            self.crit_multiplier = self.hit_multiplier;
        }

        if self.crit_chance < 0 {
            self.crit_chance = 0;
        }

        if self.hit_threshold < self.graze_threshold {
            self.graze_threshold = self.hit_threshold;
        }

        self.flanking_angle += rules.base_flanking_angle;
        self.crit_chance += rules.crit_chance as i32;
        self.hit_threshold += rules.hit_percentile as i32;
        self.graze_threshold += rules.graze_percentile as i32;
        self.graze_multiplier += rules.graze_damage_multiplier;
        self.hit_multiplier += 1.0;
        self.crit_multiplier += rules.crit_damage_multiplier;
        self.movement_rate += actor.race.movement_rate;
        self.attack_cost += rules.attack_ap as i32;

        let size_bonus = actor.race.size.diagonal / 2.0;
        self.touch_range = self.bonus_reach + size_bonus;
        self.attack_range += size_bonus;
    }
}
