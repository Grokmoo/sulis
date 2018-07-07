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
use rand::{self, Rng};

use sulis_core::image::Image;
use {Armor, AttributeList, Attack, Damage, HitKind, WeaponKind, ArmorKind, Slot};
use bonus::{AttackBonuses, AttackBuilder, BonusKind, BonusList};

#[derive(Clone)]
pub struct StatList {
    attack_range: f32,

    pub attributes: AttributeList,
    armor_proficiencies: Vec<ArmorKind>,
    weapon_proficiencies: Vec<WeaponKind>,

    // contingent bonuses are accumulated here and then applied if applicable when finalizing
    pub contingent_bonuses: BonusList,

    // these bonuses are applied only to the attack itself of the given weaponkind
    pub attack_bonuses: Vec<(WeaponKind, BonusKind)>,
    pub bonus_ap: i32,
    pub bonus_damage: Vec<Damage>,
    pub bonus_reach: f32,
    pub bonus_range: f32,
    pub attacks: Vec<Attack>,
    pub armor: Armor,
    pub max_hp: i32,
    pub initiative: i32,
    pub accuracy: i32,
    pub defense: i32,
    pub fortitude: i32,
    pub reflex: i32,
    pub will: i32,
    pub concealment: i32,
    pub concealment_ignore: i32,
    pub crit_threshold: i32,
    pub hit_threshold: i32,
    pub graze_threshold: i32,
    pub graze_multiplier: f32,
    pub hit_multiplier: f32,
    pub crit_multiplier: f32,
    pub movement_rate: f32,
    pub attack_cost: i32,
    pub move_disabled: bool,
    pub attack_disabled: bool,
    group_uses_per_encounter: HashMap<String, u32>,
}

impl StatList {
    pub fn new(attrs: AttributeList) -> StatList {
        StatList {
            attributes: attrs,
            armor_proficiencies: Vec::new(),
            weapon_proficiencies: Vec::new(),
            contingent_bonuses: BonusList::default(),
            attack_bonuses: Vec::new(),
            bonus_ap: 0,
            bonus_damage: Vec::new(),
            bonus_reach: 0.0,
            bonus_range: 0.0,
            attack_range: 0.0,
            attacks: Vec::new(),
            armor: Armor::default(),
            max_hp: 0,
            initiative: 0,
            accuracy: 0,
            defense: 0,
            fortitude: 0,
            reflex: 0,
            will: 0,
            concealment: 0,
            concealment_ignore: 0,
            crit_threshold: 0,
            hit_threshold: 0,
            graze_threshold: 0,
            graze_multiplier: 0.0,
            hit_multiplier: 0.0,
            crit_multiplier: 0.0,
            movement_rate: 0.0,
            attack_cost: 0,
            move_disabled: false,
            attack_disabled: false,
            group_uses_per_encounter: HashMap::new(),
        }
    }

    pub fn uses_per_encounter_iter(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.group_uses_per_encounter.iter()
    }

    pub fn ability_groups(&self) -> Vec<String> {
        self.group_uses_per_encounter.keys().map(|k| k.to_string()).collect()
    }

    pub fn uses_per_encounter(&self, ability_group: &str) -> u32 {
        *self.group_uses_per_encounter.get(ability_group).unwrap_or(&0)
    }

    pub fn has_armor_proficiency(&self, prof: ArmorKind) -> bool {
        self.armor_proficiencies.contains(&prof)
    }

    pub fn has_weapon_proficiency(&self, prof: WeaponKind) -> bool {
        self.weapon_proficiencies.contains(&prof)
    }

    pub fn attack_roll(&self, defense: i32, bonuses: &AttackBonuses) -> HitKind {
        let accuracy = self.accuracy + bonuses.accuracy;
        let roll = rand::thread_rng().gen_range(1, 101);
        debug!("Attack roll: {} with accuracy {} against {}", roll, accuracy, defense);

        if roll + accuracy < defense { return HitKind::Miss; }

        let result = roll + accuracy - defense;

        if result > self.crit_threshold + bonuses.crit_threshold {
            HitKind::Crit
        } else if result > self.hit_threshold + bonuses.hit_threshold {
            HitKind::Hit
        } else if result > self.graze_threshold + bonuses.graze_threshold {
            HitKind::Graze
        } else {
            HitKind::Miss
        }
    }

    pub fn attack_is_melee(&self) -> bool {
        if self.attacks.is_empty() { return false; }

        self.attacks[0].is_melee()
    }

    pub fn attack_is_ranged(&self) -> bool {
        if self.attacks.is_empty() { return false; }

        self.attacks[0].is_ranged()
    }

    pub fn get_ranged_projectile(&self) -> Option<Rc<Image>> {
        if !self.attack_is_ranged() { return None; }

        self.attacks[0].get_ranged_projectile()
    }

    /// Returns the maximum distance that this StatList's
    /// attacks can reach
    pub fn attack_distance(&self) -> f32 {
        self.attack_range
    }

    pub fn add_single_group_uses_per_encounter(&mut self, group_id: &str, uses: u32) {
        *self.group_uses_per_encounter.entry(group_id.to_string()).or_insert(0) += uses;
    }

    pub fn add_group_uses_per_encounter(&mut self, uses: &Vec<(String, u32)>, times: u32) {
        for (ref group_id, amount) in uses.iter() {
            *self.group_uses_per_encounter.entry(group_id.to_string()).or_insert(0) += amount * times;
        }
    }

    /// Adds the bonuses from the specified BonusList to this stat list.
    pub fn add(&mut self, bonuses: &BonusList) {
        self.add_multiple(bonuses, 1);
    }

    /// Adds the specified bonuses to this StatList the specified number of times.
    /// Note that non-numeric bonuses are only added once regardless of the value of
    /// times
    pub fn add_multiple(&mut self, bonuses: &BonusList, times: u32) {
        if times == 0 { return; }

        // TODO handle add multiple for weapon and attack bonuses
        for bonus in bonuses.iter() {
            use bonus::Contingent::*;
            match bonus.when {
                Always => self.add_bonus(&bonus.kind, times),
                AttackWithWeapon(weapon_kind) => self.attack_bonuses.push((weapon_kind, bonus.kind.clone())),
                WeaponEquipped(_) | ArmorEquipped {..} => self.contingent_bonuses.add(bonus.clone()),
            }
        }
    }

    fn add_bonus(&mut self, bonus: &BonusKind, times: u32) {
        let times_i32 = times as i32;
        let times_f32 = times as f32;

        use bonus::BonusKind::*;
        match bonus {
            Attribute { attribute, amount } => self.attributes.add(*attribute, *amount),
            ActionPoints(amount) => self.bonus_ap += amount * times_i32,
            Armor(amount) => self.armor.add_base(amount * times_i32),
            ArmorKind { kind, amount } => self.armor.add_kind(*kind, amount * times_i32),
            Damage(damage) => self.bonus_damage.push(damage.mult(times)),
            ArmorProficiency(kind) => {
                if !self.armor_proficiencies.contains(kind) {
                    self.armor_proficiencies.push(*kind);
                }
            },
            WeaponProficiency(kind) => {
                if !self.weapon_proficiencies.contains(kind) {
                    self.weapon_proficiencies.push(*kind);
                }
            },
            Reach(amount) => self.bonus_reach += amount * times_f32,
            Range(amount) => self.bonus_range += amount * times_f32,
            Initiative(amount) => self.initiative += amount * times_i32,
            HitPoints(amount) => self.max_hp += amount * times_i32,
            Accuracy(amount) => self.accuracy += amount * times_i32,
            Defense(amount) => self.defense += amount * times_i32,
            Fortitude(amount) => self.fortitude += amount * times_i32,
            Reflex(amount) => self.reflex += amount * times_i32,
            Will(amount) => self.will += amount * times_i32,
            Concealment(amount) => self.concealment += amount * times_i32,
            ConcealmentIgnore(amount) => self.concealment_ignore += amount * times_i32,
            CritThreshold(amount) => self.crit_threshold += amount * times_i32,
            HitThreshold(amount) => self.hit_threshold += amount * times_i32,
            GrazeThreshold(amount) => self.graze_threshold += amount * times_i32,
            CritMultiplier(amount) => self.crit_multiplier += amount * times_f32,
            HitMultiplier(amount) => self.hit_multiplier += amount * times_f32,
            GrazeMultiplier(amount) => self.graze_multiplier += amount * times_f32,
            MovementRate(amount) => self.movement_rate += amount * times_f32,
            AttackCost(amount) => self.attack_cost += amount * times_i32,
            MoveDisabled => self.move_disabled = true,
            AttackDisabled => self.attack_disabled = true,
            GroupUsesPerEncounter { group, amount } => self.add_single_group_uses_per_encounter(group, *amount),
        }
    }

    pub fn finalize(&mut self,
                    attacks: Vec<(&AttackBuilder, WeaponKind)>,
                    equipped_armor: HashMap<Slot, ArmorKind>,
                    multiplier: f32,
                    base_attr: i32) {
        if attacks.is_empty() {
            warn!("Finalized stats with no attacks");
        }

        // clone here to avoid problem with add_bonus needing mutable borrow,
        // even though it would be safe
        let contingent = self.contingent_bonuses.clone();
        for bonus in contingent.iter() {
            use bonus::Contingent::*;
            match bonus.when {
                Always | AttackWithWeapon(_) => unreachable!(),
                WeaponEquipped(weapon_kind) => {
                    for (_, attack_weapon_kind) in attacks.iter() {
                        if weapon_kind == *attack_weapon_kind { self.add_bonus(&bonus.kind, 1); }
                    }
                }
                ArmorEquipped { kind, slot } => {
                    if let Some(armor_kind) = equipped_armor.get(&slot) {
                        if *armor_kind == kind {
                            self.add_bonus(&bonus.kind, 1);
                        }
                    }
                }
            }
        }

        let mut attack_range = None;
        for (builder, weapon_kind) in attacks {
            let mut attack = Attack::new(builder, &self, weapon_kind).mult(multiplier);

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

        use Attribute::*;
        let attrs = &self.attributes;
        self.initiative += attrs.bonus(Perception, base_attr) / 2;
        self.accuracy += attrs.bonus(Perception, base_attr);
        self.defense += attrs.bonus(Dexterity, base_attr);
        self.fortitude += attrs.bonus(Endurance, base_attr);
        self.reflex += attrs.bonus(Dexterity, base_attr);
        self.will += attrs.bonus(Wisdom, base_attr);

        if self.crit_threshold < self.hit_threshold {
            self.hit_threshold = self.crit_threshold;
        }

        if self.hit_threshold < self.graze_threshold {
            self.graze_threshold = self.hit_threshold;
        }
    }
}

