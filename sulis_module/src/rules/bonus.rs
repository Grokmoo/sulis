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

use std::mem;

use crate::rules::{ArmorKind, Attribute, Damage, DamageKind, Slot, WeaponKind, WeaponStyle};
use sulis_core::util::ExtInt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum BonusKind {
    Attribute { attribute: Attribute, amount: i8 },
    ActionPoints(i32),
    Armor(i32),
    ArmorKind { kind: DamageKind, amount: i32 },
    Resistance { kind: DamageKind, amount: i32 },
    Damage(Damage),
    ArmorProficiency(ArmorKind),
    WeaponProficiency(WeaponKind),
    Reach(f32),
    Range(f32),
    Initiative(i32),
    HitPoints(i32),
    MeleeAccuracy(i32),
    RangedAccuracy(i32),
    SpellAccuracy(i32),
    Defense(i32),
    Fortitude(i32),
    Reflex(i32),
    Will(i32),
    Concealment(i32),
    ConcealmentIgnore(i32),
    CritChance(i32),
    HitThreshold(i32),
    GrazeThreshold(i32),
    CritMultiplier(f32),
    HitMultiplier(f32),
    GrazeMultiplier(f32),
    MovementRate(f32),
    AttackCost(i32),
    FlankingAngle(i32),
    CasterLevel(i32),
    AbilityActionPointCost(i32),
    FreeAbilityGroupUse,
    MoveDisabled,
    AttackDisabled,
    AbilitiesDisabled,
    Hidden,
    FlankedImmunity,
    SneakAttackImmunity,
    CritImmunity,
    GroupUsesPerEncounter { group: String, amount: ExtInt },
    GroupUsesPerDay { group: String, amount: ExtInt },
    ClassStat { id: String, amount: i32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum Contingent {
    /// Bonuses that should always be applied
    Always,

    /// Bonuses that should only be applied to the parent if they have the given
    /// WeaponKind equipped
    WeaponEquipped(WeaponKind),

    /// Bonuses that should only be applied to the parent if they have a shield of
    /// the specified type equipped
    ArmorEquipped { kind: ArmorKind, slot: Slot },

    /// Bonus that should only be applied if the parent's attack is of the given
    /// weapon style
    WeaponStyle(WeaponStyle),

    /// For bonuses applied to attacks, only Damage, MeleeAccuracy, RangedAccuracy,
    /// SpellAccuracy, CritChance, HitThreshold,
    /// GrazeThreshold, CritMultiplier, HitMultiplier, and GrazeMultiplier are valid

    /// Bonuses that should only be applied to an attack using the given WeaponKind
    AttackWithWeapon(WeaponKind),

    /// Bonuses that are only applied to attacks when the attacker is hidden
    AttackWhenHidden,

    /// Bonuses that are only applied to attacks when the attacker is flanking
    AttackWhenFlanking,

    /// Bonuses that are only applied to attacks with the specified base damage kind
    AttackWithDamageKind(DamageKind),

    /// Bonuses that only apply when the parent is threatened in melee
    Threatened,
}

impl Default for Contingent {
    fn default() -> Contingent {
        Contingent::Always
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bonus {
    #[serde(default)]
    pub when: Contingent,
    pub kind: BonusKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BonusList(Vec<Bonus>);

impl Default for BonusList {
    fn default() -> BonusList {
        BonusList(Vec::new())
    }
}

impl BonusList {
    pub fn iter(&self) -> impl Iterator<Item = &Bonus> {
        self.0.iter()
    }

    pub fn add(&mut self, bonus: Bonus) {
        self.0.push(bonus);
    }

    pub fn add_kind(&mut self, kind: BonusKind) {
        self.0.push(Bonus {
            when: Contingent::Always,
            kind,
        });
    }

    pub fn merge_duplicates(&mut self) {
        let bonuses = &mut self.0;
        bonuses.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut i = 0;
        loop {
            if i + 1 >= bonuses.len() {
                return;
            }

            if let Some(merged_bonus) = merge_if_dup(&bonuses[i], &bonuses[i + 1]) {
                mem::replace(&mut bonuses[i], merged_bonus);
                bonuses.remove(i + 1);
            }

            i += 1;
        }
    }

    pub fn apply_modifiers(&mut self, penalty_mod: f32, bonus_mod: f32) {
        for mut bonus in self.0.iter_mut() {
            apply_modifiers(&mut bonus, penalty_mod, bonus_mod);
        }
    }
}

macro_rules! apply_kind_mod_i32 {
    ($kind:ident ( $val:ident ) : $penalty:ident, $bonus:ident) => {
        if $val > 0 {
            $kind(($val as f32 * $bonus).round() as i32)
        } else {
            $kind(($val as f32 * $penalty).round() as i32)
        }
    };
}

macro_rules! apply_kind_mod_f32 {
    ($kind:ident ( $val:ident ) : $penalty:ident, $bonus:ident) => {
        if $val > 0.0 {
            $kind($val * $bonus)
        } else {
            $kind($val * $penalty)
        }
    };
}

fn apply_modifiers(bonus: &mut Bonus, neg: f32, pos: f32) {
    use self::BonusKind::*;
    let new_kind = match bonus.kind {
        // all of these could easily merged into one macro, but then it
        // would need a separate match and we would lose the exhaustiveness check
        ActionPoints(val) => apply_kind_mod_i32!(ActionPoints(val): neg, pos),
        Armor(val) => apply_kind_mod_i32!(Armor(val): neg, pos),
        Reach(val) => apply_kind_mod_f32!(Reach(val): neg, pos),
        Range(val) => apply_kind_mod_f32!(Range(val): neg, pos),
        Initiative(val) => apply_kind_mod_i32!(Initiative(val): neg, pos),
        HitPoints(val) => apply_kind_mod_i32!(HitPoints(val): neg, pos),
        MeleeAccuracy(val) => apply_kind_mod_i32!(MeleeAccuracy(val): neg, pos),
        RangedAccuracy(val) => apply_kind_mod_i32!(RangedAccuracy(val): neg, pos),
        SpellAccuracy(val) => apply_kind_mod_i32!(SpellAccuracy(val): neg, pos),
        Defense(val) => apply_kind_mod_i32!(Defense(val): neg, pos),
        Fortitude(val) => apply_kind_mod_i32!(Fortitude(val): neg, pos),
        Reflex(val) => apply_kind_mod_i32!(Reflex(val): neg, pos),
        Will(val) => apply_kind_mod_i32!(Will(val): neg, pos),
        Concealment(val) => apply_kind_mod_i32!(Concealment(val): neg, pos),
        ConcealmentIgnore(val) => apply_kind_mod_i32!(ConcealmentIgnore(val): neg, pos),
        CritChance(val) => apply_kind_mod_i32!(CritChance(val): neg, pos),
        HitThreshold(val) => apply_kind_mod_i32!(HitThreshold(val): neg, pos),
        GrazeThreshold(val) => apply_kind_mod_i32!(GrazeThreshold(val): neg, pos),
        CritMultiplier(val) => apply_kind_mod_f32!(CritMultiplier(val): neg, pos),
        HitMultiplier(val) => apply_kind_mod_f32!(HitMultiplier(val): neg, pos),
        GrazeMultiplier(val) => apply_kind_mod_f32!(GrazeMultiplier(val): neg, pos),
        MovementRate(val) => apply_kind_mod_f32!(MovementRate(val): neg, pos),
        AttackCost(val) => apply_kind_mod_i32!(AttackCost(val): neg, pos),
        FlankingAngle(val) => apply_kind_mod_i32!(FlankingAngle(val): neg, pos),
        CasterLevel(val) => apply_kind_mod_i32!(CasterLevel(val): neg, pos),
        AbilityActionPointCost(val) => apply_kind_mod_i32!(AbilityActionPointCost(val): neg, pos),
        Damage(damage) => Damage(damage.mult_f32(pos)),
        ClassStat { ref id, amount } => {
            if amount > 0 {
                ClassStat { id: id.clone(), amount: (amount as f32 * pos).round() as i32 }
            } else {
                ClassStat { id: id.clone(), amount: (amount as f32 * neg).round() as i32 }
            }
        },
        ArmorKind { kind, amount } => {
            if amount > 0 {
                ArmorKind {
                    kind,
                    amount: (amount as f32 * pos).round() as i32,
                }
            } else {
                ArmorKind {
                    kind,
                    amount: (amount as f32 * neg).round() as i32,
                }
            }
        }
        Resistance { kind, amount } => {
            if amount > 0 {
                Resistance {
                    kind,
                    amount: (amount as f32 * pos).round() as i32,
                }
            } else {
                Resistance {
                    kind,
                    amount: (amount as f32 * neg).round() as i32,
                }
            }
        }
        Attribute { attribute, amount } => {
            if amount > 0 {
                Attribute {
                    attribute,
                    amount: (amount as f32 * pos).round() as i8,
                }
            } else {
                Attribute {
                    attribute,
                    amount: (amount as f32 * neg).round() as i8,
                }
            }
        }
        ArmorProficiency(_)
        | WeaponProficiency(_)
        | MoveDisabled
        | AttackDisabled
        | Hidden
        | GroupUsesPerEncounter { .. }
        | GroupUsesPerDay { .. }
        | FlankedImmunity
        | SneakAttackImmunity
        | CritImmunity
        | AbilitiesDisabled
        | FreeAbilityGroupUse => return,
    };

    bonus.kind = new_kind;
}

macro_rules! merge_int_bonus {
    ($kind:ident, $val:ident, $sec:ident, $when:ident) => {
        if let $kind(other) = $sec.kind {
            return Some(Bonus {
                $when,
                kind: $kind($val + other),
            });
        }
    };
}

pub fn merge_if_dup(first: &Bonus, sec: &Bonus) -> Option<Bonus> {
    if first.when != sec.when {
        return None;
    }

    let when = first.when;
    use self::BonusKind::*;
    match first.kind {
        Attribute { attribute, amount } => {
            if let Attribute {
                attribute: attr,
                amount: amt,
            } = sec.kind
            {
                if attr != attribute {
                    return None;
                }
                return Some(Bonus {
                    when,
                    kind: Attribute {
                        attribute,
                        amount: amount + amt,
                    },
                });
            }
        }
        ArmorKind { kind, amount } => {
            if let ArmorKind {
                kind: other_kind,
                amount: other,
            } = sec.kind
            {
                if other_kind != kind {
                    return None;
                }
                return Some(Bonus {
                    when,
                    kind: ArmorKind {
                        kind,
                        amount: amount + other,
                    },
                });
            }
        }
        Resistance { kind, amount } => {
            if let Resistance {
                kind: other_kind,
                amount: other,
            } = sec.kind
            {
                if other_kind != kind {
                    return None;
                }
                return Some(Bonus {
                    when,
                    kind: Resistance {
                        kind,
                        amount: amount + other,
                    },
                });
            }
        }
        Damage(damage) => {
            if let Damage(other) = sec.kind {
                if damage.kind != other.kind {
                    return None;
                }
                let mut damage = damage.clone();
                damage.add(other);
                return Some(Bonus {
                    when,
                    kind: Damage(damage),
                });
            }
        }
        ArmorProficiency(kind) => {
            if let ArmorProficiency(other) = sec.kind {
                if kind != other {
                    return None;
                }
                return Some(Bonus {
                    when,
                    kind: ArmorProficiency(kind),
                });
            }
        }
        WeaponProficiency(kind) => {
            if let WeaponProficiency(other) = sec.kind {
                if kind != other {
                    return None;
                }
                return Some(Bonus {
                    when,
                    kind: WeaponProficiency(kind),
                });
            }
        }
        MoveDisabled => {
            if let MoveDisabled = sec.kind {
                return Some(Bonus {
                    when,
                    kind: MoveDisabled,
                });
            }
        }
        AbilitiesDisabled => {
            if let AbilitiesDisabled = sec.kind {
                return Some(Bonus {
                    when,
                    kind: AbilitiesDisabled,
                });
            }
        }
        AttackDisabled => {
            if let AttackDisabled = sec.kind {
                return Some(Bonus {
                    when,
                    kind: AttackDisabled,
                });
            }
        }
        Hidden => {
            if let Hidden = sec.kind {
                return Some(Bonus { when, kind: Hidden });
            }
        }
        FlankedImmunity => {
            if let FlankedImmunity = sec.kind {
                return Some(Bonus {
                    when,
                    kind: FlankedImmunity,
                });
            }
        }
        SneakAttackImmunity => {
            if let SneakAttackImmunity = sec.kind {
                return Some(Bonus {
                    when,
                    kind: SneakAttackImmunity,
                });
            }
        }
        CritImmunity => {
            if let CritImmunity = sec.kind {
                return Some(Bonus {
                    when,
                    kind: CritImmunity,
                });
            }
        }
        FreeAbilityGroupUse => {
            if let FreeAbilityGroupUse = sec.kind {
                return Some(Bonus {
                    when,
                    kind: FreeAbilityGroupUse,
                });
            }
        }
        GroupUsesPerEncounter { ref group, amount } => {
            if let GroupUsesPerEncounter {
                group: ref other_grp,
                amount: amt,
            } = sec.kind
            {
                if group != other_grp {
                    return None;
                }
                let group = group.clone();
                let amount = amount + amt;
                return Some(Bonus {
                    when,
                    kind: GroupUsesPerEncounter { group, amount },
                });
            }
        }
        GroupUsesPerDay { ref group, amount } => {
            if let GroupUsesPerDay {
                group: ref other_grp,
                amount: amt,
            } = sec.kind
            {
                if group != other_grp {
                    return None;
                }
                let group = group.clone();
                let amount = amount + amt;
                return Some(Bonus {
                    when,
                    kind: GroupUsesPerDay { group, amount },
                });
            }
        }
        ClassStat { ref id, amount } => {
            if let ClassStat { id: ref other_id, amount: amt } = sec.kind {
                if id != other_id { return None; }
                let id = id.clone();
                let amount = amount + amt;
                return Some(Bonus { when, kind: ClassStat { id, amount } });
            }
        }
        // all of these statements could be easily merged into one macro,
        // but then it would need to be its own match statement and you would
        // lose the exhaustiveness check
        AbilityActionPointCost(val) => merge_int_bonus!(AbilityActionPointCost, val, sec, when),
        ActionPoints(val) => merge_int_bonus!(ActionPoints, val, sec, when),
        Armor(val) => merge_int_bonus!(Armor, val, sec, when),
        Range(val) => merge_int_bonus!(Range, val, sec, when),
        Reach(val) => merge_int_bonus!(Reach, val, sec, when),
        Initiative(val) => merge_int_bonus!(Initiative, val, sec, when),
        HitPoints(val) => merge_int_bonus!(HitPoints, val, sec, when),
        MeleeAccuracy(val) => merge_int_bonus!(MeleeAccuracy, val, sec, when),
        RangedAccuracy(val) => merge_int_bonus!(RangedAccuracy, val, sec, when),
        SpellAccuracy(val) => merge_int_bonus!(SpellAccuracy, val, sec, when),
        Defense(val) => merge_int_bonus!(Defense, val, sec, when),
        Fortitude(val) => merge_int_bonus!(Fortitude, val, sec, when),
        Reflex(val) => merge_int_bonus!(Reflex, val, sec, when),
        Will(val) => merge_int_bonus!(Will, val, sec, when),
        Concealment(val) => merge_int_bonus!(Concealment, val, sec, when),
        ConcealmentIgnore(val) => merge_int_bonus!(ConcealmentIgnore, val, sec, when),
        CritChance(val) => merge_int_bonus!(CritChance, val, sec, when),
        HitThreshold(val) => merge_int_bonus!(HitThreshold, val, sec, when),
        GrazeThreshold(val) => merge_int_bonus!(GrazeThreshold, val, sec, when),
        CritMultiplier(val) => merge_int_bonus!(CritMultiplier, val, sec, when),
        HitMultiplier(val) => merge_int_bonus!(HitMultiplier, val, sec, when),
        GrazeMultiplier(val) => merge_int_bonus!(GrazeMultiplier, val, sec, when),
        MovementRate(val) => merge_int_bonus!(MovementRate, val, sec, when),
        AttackCost(val) => merge_int_bonus!(AttackCost, val, sec, when),
        FlankingAngle(val) => merge_int_bonus!(FlankingAngle, val, sec, when),
        CasterLevel(val) => merge_int_bonus!(CasterLevel, val, sec, when),
    }

    None
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct AttackBonuses {
    pub damage: Option<Damage>,
    pub melee_accuracy: i32,
    pub ranged_accuracy: i32,
    pub spell_accuracy: i32,
    pub crit_chance: i32,
    pub hit_threshold: i32,
    pub graze_threshold: i32,
    pub crit_multiplier: f32,
    pub hit_multiplier: f32,
    pub graze_multiplier: f32,
}

macro_rules! mod_field {
    ($field:expr, $pos:ident, $neg:ident) => {
        if $field > 0 {
            $field = ($field as f32 * $pos).round() as i32;
        } else {
            $field = ($field as f32 * $neg).round() as i32;
        }
    };
}

macro_rules! mod_field_f32 {
    ($field:expr, $pos:ident, $neg:ident) => {
        if $field > 0.0 {
            $field = $field * $pos;
        } else {
            $field = $field as f32 * $neg;
        }
    };
}

impl AttackBonuses {
    pub fn add(&mut self, other: &AttackBonuses) {
        if let Some(mut damage) = self.damage {
            if let Some(other) = other.damage {
                damage.add(other.clone());
            }
        } else {
            if let Some(other) = other.damage {
                self.damage = Some(other.clone());
            }
        }

        self.melee_accuracy += other.melee_accuracy;
        self.ranged_accuracy += other.ranged_accuracy;
        self.spell_accuracy += other.spell_accuracy;
        self.crit_chance += other.crit_chance;
        self.hit_threshold += other.hit_threshold;
        self.graze_threshold += other.graze_threshold;
        self.crit_multiplier += other.crit_multiplier;
        self.hit_multiplier += other.hit_multiplier;
        self.graze_multiplier += other.graze_multiplier;
    }

    pub fn apply_modifier(&mut self, neg: f32, pos: f32) {
        if let Some(mut damage) = self.damage {
            damage.mult_f32_mut(pos);
        }

        mod_field!(self.melee_accuracy, pos, neg);
        mod_field!(self.ranged_accuracy, pos, neg);
        mod_field!(self.spell_accuracy, pos, neg);
        mod_field!(self.crit_chance, pos, neg);
        mod_field!(self.hit_threshold, pos, neg);
        mod_field!(self.graze_threshold, pos, neg);
        mod_field_f32!(self.crit_multiplier, pos, neg);
        mod_field_f32!(self.hit_multiplier, pos, neg);
        mod_field_f32!(self.graze_multiplier, pos, neg);
    }
}

impl Default for AttackBonuses {
    fn default() -> AttackBonuses {
        AttackBonuses {
            damage: None,
            melee_accuracy: 0,
            ranged_accuracy: 0,
            spell_accuracy: 0,
            crit_chance: 0,
            hit_threshold: 0,
            graze_threshold: 0,
            crit_multiplier: 0.0,
            hit_multiplier: 0.0,
            graze_multiplier: 0.0,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackBuilder {
    pub damage: Damage,
    pub kind: AttackKindBuilder,
    pub bonuses: AttackBonuses,
}

impl AttackBuilder {
    pub fn distance(&self) -> f32 {
        match self.kind {
            AttackKindBuilder::Melee { reach } => reach,
            AttackKindBuilder::Ranged { range, .. } => range,
        }
    }

    pub fn mult(&mut self, multiplier: f32) -> AttackBuilder {
        AttackBuilder {
            damage: self.damage.mult_f32(multiplier),
            kind: self.kind.clone(),
            bonuses: self.bonuses.clone(),
        }
    }

    pub fn is_melee(&self) -> bool {
        match self.kind {
            AttackKindBuilder::Melee { .. } => true,
            _ => false,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum AttackKindBuilder {
    Melee { reach: f32 },
    Ranged { range: f32, projectile: String },
}
