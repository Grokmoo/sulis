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

use crate::rules::{
    ArmorKind, Attribute, Damage, DamageKind, HitKind, Slot, WeaponKind, WeaponStyle
};
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
    MoveAnimRate(f32),
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
                let _ = mem::replace(&mut bonuses[i], merged_bonus);
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

macro_rules! get_mod {
    ($val:expr, f32, $penalty:ident, $bonus:ident) => {
        $val * if $val >= 0.0 {$bonus} else {$penalty}
    };
    ($val:expr, $tp:ty, $penalty:ident, $bonus:ident) => {
        if $val >= 0 {
            ($val as f32 * $bonus).round() as $tp
        } else {
            ($val as f32 * $penalty).round() as $tp
        }
    };
    ($variant:ident ( $val:ident ): $($tail:tt)*) => {
        $variant(get_mod!($val, $($tail)*))
    };
    ($variant:ident {$kind:ident, $amount:ident}: $($tail:tt)*) => {
        $variant {
            $kind,
            $amount: get_mod!($amount, $($tail)*)
        }
    };
}

#[allow(clippy::cognitive_complexity)]
fn apply_modifiers(bonus: &mut Bonus, neg: f32, pos: f32) {
    use self::BonusKind::*;
    let new_kind = match bonus.kind {
        // all of these could easily merged into one macro, but then it
        // would need a separate match and we would lose the exhaustiveness check
        ActionPoints(val) => get_mod!(ActionPoints(val): i32, neg, pos),
        Armor(val) => get_mod!(Armor(val): i32, neg, pos),
        Reach(val) => get_mod!(Reach(val): f32, neg, pos),
        Range(val) => get_mod!(Range(val): f32, neg, pos),
        Initiative(val) => get_mod!(Initiative(val): i32, neg, pos),
        HitPoints(val) => get_mod!(HitPoints(val): i32, neg, pos),
        MeleeAccuracy(val) => get_mod!(MeleeAccuracy(val): i32, neg, pos),
        RangedAccuracy(val) => get_mod!(RangedAccuracy(val): i32, neg, pos),
        SpellAccuracy(val) => get_mod!(SpellAccuracy(val): i32, neg, pos),
        Defense(val) => get_mod!(Defense(val): i32, neg, pos),
        Fortitude(val) => get_mod!(Fortitude(val): i32, neg, pos),
        Reflex(val) => get_mod!(Reflex(val): i32, neg, pos),
        Will(val) => get_mod!(Will(val): i32, neg, pos),
        Concealment(val) => get_mod!(Concealment(val): i32, neg, pos),
        ConcealmentIgnore(val) => get_mod!(ConcealmentIgnore(val): i32, neg, pos),
        CritChance(val) => get_mod!(CritChance(val): i32, neg, pos),
        HitThreshold(val) => get_mod!(HitThreshold(val): i32, neg, pos),
        GrazeThreshold(val) => get_mod!(GrazeThreshold(val): i32, neg, pos),
        CritMultiplier(val) => get_mod!(CritMultiplier(val): f32, neg, pos),
        HitMultiplier(val) => get_mod!(HitMultiplier(val): f32, neg, pos),
        GrazeMultiplier(val) => get_mod!(GrazeMultiplier(val): f32, neg, pos),
        MovementRate(val) => get_mod!(MovementRate(val): f32, neg, pos),
        MoveAnimRate(val) => get_mod!(MoveAnimRate(val): f32, neg, pos),
        AttackCost(val) => get_mod!(AttackCost(val): i32, neg, pos),
        FlankingAngle(val) => get_mod!(FlankingAngle(val): i32, neg, pos),
        CasterLevel(val) => get_mod!(CasterLevel(val): i32, neg, pos),
        AbilityActionPointCost(val) => get_mod!(AbilityActionPointCost(val): i32, neg, pos),
        Damage(damage) => Damage(damage.mult_f32(pos)),
        ClassStat { ref id, amount } => ClassStat {
            id: id.clone(),
            amount: get_mod!(amount, i32, pos, neg),
        },
        ArmorKind { kind, amount } => get_mod!(ArmorKind { kind, amount }: i32, neg, pos),
        Resistance { kind, amount } => get_mod!(Resistance { kind, amount }: i32, neg, pos),
        Attribute { attribute, amount } => get_mod!(Attribute { attribute, amount }: i8, neg, pos),
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

macro_rules! merge_dup {
    ($when:ident, $variant:ident {$attr:ident, $amount:ident}: $amt:ident) => {
        Some(Bonus {
            $when,
            kind: $variant {
                $attr,
                $amount: $amount + $amt,
            },
        })
    };
    ($when:ident, $variant:ident($amount:ident): $other:ident) => {
        Some(Bonus {
            $when,
            kind: $variant($amount + $other),
        })
    };
    ($variant:ident: $sec:ident, $when:ident) => {
        match $sec.kind {
            $variant => Some(Bonus {
                $when,
                kind: $variant,
            }),
            _ => None,
        }
    };
    ($variant:ident($amount:ident): $sec:ident, $when:ident) => {
        match $sec.kind {
            $variant(other) => merge_dup!($when, $variant($amount): other),
            _ => None,
        }
    };
    ($variant:ident($kind:ident): $sec:ident, $test:ident, $when:ident) => {
        match $sec.kind {
            $variant($test) if $kind == $test => Some(Bonus {
                $when,
                kind: $variant($kind),
            }),
            _ => None,
        }
    };
    ($variant:ident {$attr:ident, $amount:ident}: $sec:ident, $when:ident) => {
        match $sec.kind {
            $variant {
                $attr: test,
                $amount: amt,
            } if $attr == test => merge_dup!($when, $variant { $attr, $amount }: amt),
            _ => None,
        }
    };
    ($variant:ident {ref $var:ident, $amount:ident}: $sec:ident, $when:ident) => {
        match $sec.kind {
            $variant {
                $var: ref other,
                $amount: amt,
            } if $var == other => {
                let $var = $var.clone();
                merge_dup!($when, $variant { $var, $amount }: amt)
            }
            _ => None,
        }
    };
}

#[allow(clippy::cognitive_complexity)]
pub fn merge_if_dup(first: &Bonus, sec: &Bonus) -> Option<Bonus> {
    if first.when != sec.when {
        return None;
    }

    let when = first.when;
    use self::BonusKind::*;
    match first.kind {
        Attribute { attribute, amount } => merge_dup!(Attribute { attribute, amount }: sec, when),
        ArmorKind { kind, amount } => merge_dup!(ArmorKind { kind, amount }: sec, when),
        Resistance { kind, amount } => merge_dup!(Resistance { kind, amount }: sec, when),
        Damage(mut damage) => match sec.kind {
            Damage(other) if damage.kind == other.kind => {
                damage.add(other);
                Some(Bonus {
                    when,
                    kind: Damage(damage),
                })
            }
            _ => None,
        },
        ArmorProficiency(kind) => merge_dup!(ArmorProficiency(kind): sec, test_name, when),
        WeaponProficiency(kind) => merge_dup!(WeaponProficiency(kind): sec, test_name, when),

        MoveDisabled => merge_dup!(MoveDisabled: sec, when),
        AbilitiesDisabled => merge_dup!(AbilitiesDisabled: sec, when),
        AttackDisabled => merge_dup!(AttackDisabled: sec, when),
        Hidden => merge_dup!(Hidden: sec, when),
        FlankedImmunity => merge_dup!(FlankedImmunity: sec, when),
        SneakAttackImmunity => merge_dup!(SneakAttackImmunity: sec, when),
        CritImmunity => merge_dup!(CritImmunity: sec, when),
        FreeAbilityGroupUse => merge_dup!(FreeAbilityGroupUse: sec, when),

        GroupUsesPerEncounter { ref group, amount } => {
            merge_dup!(GroupUsesPerEncounter{ref group, amount}: sec, when)
        }
        GroupUsesPerDay { ref group, amount } => {
            merge_dup!(GroupUsesPerDay{ref group, amount}: sec, when)
        }
        ClassStat { ref id, amount } => merge_dup!(ClassStat{ref id, amount}: sec, when),

        AbilityActionPointCost(val) => merge_dup!(AbilityActionPointCost(val): sec, when),
        ActionPoints(val) => merge_dup!(ActionPoints(val): sec, when),
        Armor(val) => merge_dup!(Armor(val): sec, when),
        Range(val) => merge_dup!(Range(val): sec, when),
        Reach(val) => merge_dup!(Reach(val): sec, when),
        Initiative(val) => merge_dup!(Initiative(val): sec, when),
        HitPoints(val) => merge_dup!(HitPoints(val): sec, when),
        MeleeAccuracy(val) => merge_dup!(MeleeAccuracy(val): sec, when),
        RangedAccuracy(val) => merge_dup!(RangedAccuracy(val): sec, when),
        SpellAccuracy(val) => merge_dup!(SpellAccuracy(val): sec, when),
        Defense(val) => merge_dup!(Defense(val): sec, when),
        Fortitude(val) => merge_dup!(Fortitude(val): sec, when),
        Reflex(val) => merge_dup!(Reflex(val): sec, when),
        Will(val) => merge_dup!(Will(val): sec, when),
        Concealment(val) => merge_dup!(Concealment(val): sec, when),
        ConcealmentIgnore(val) => merge_dup!(ConcealmentIgnore(val): sec, when),
        CritChance(val) => merge_dup!(CritChance(val): sec, when),
        HitThreshold(val) => merge_dup!(HitThreshold(val): sec, when),
        GrazeThreshold(val) => merge_dup!(GrazeThreshold(val): sec, when),
        CritMultiplier(val) => merge_dup!(CritMultiplier(val): sec, when),
        HitMultiplier(val) => merge_dup!(HitMultiplier(val): sec, when),
        GrazeMultiplier(val) => merge_dup!(GrazeMultiplier(val): sec, when),
        MovementRate(val) => merge_dup!(MovementRate(val): sec, when),
        MoveAnimRate(val) => merge_dup!(MoveAnimRate(val): sec, when),
        AttackCost(val) => merge_dup!(AttackCost(val): sec, when),
        FlankingAngle(val) => merge_dup!(FlankingAngle(val): sec, when),
        CasterLevel(val) => merge_dup!(CasterLevel(val): sec, when),
    }
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

impl AttackBonuses {
    pub fn add(&mut self, other: &AttackBonuses) {
        if let Some(mut damage) = self.damage {
            if let Some(other) = other.damage {
                damage.add(other);
            }
        } else if let Some(other) = other.damage {
            self.damage = Some(other);
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

        self.melee_accuracy = get_mod!(self.melee_accuracy, i32, neg, pos);
        self.ranged_accuracy = get_mod!(self.ranged_accuracy, i32, neg, pos);
        self.spell_accuracy = get_mod!(self.spell_accuracy, i32, neg, pos);
        self.crit_chance = get_mod!(self.crit_chance, i32, neg, pos);
        self.hit_threshold = get_mod!(self.hit_threshold, i32, neg, pos);
        self.graze_threshold = get_mod!(self.graze_threshold, i32, neg, pos);
        self.crit_multiplier = get_mod!(self.crit_multiplier, f32, neg, pos);
        self.hit_multiplier = get_mod!(self.hit_multiplier, f32, neg, pos);
        self.graze_multiplier = get_mod!(self.graze_multiplier, f32, neg, pos);
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

    #[serde(default)]
    pub bonuses: AttackBonuses,

    #[serde(default)]
    pub sounds: HitSounds,
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
            sounds: self.sounds.clone(),
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
#[serde(deny_unknown_fields)]
pub struct HitSounds {
    miss: Option<String>,
    graze: Option<String>,
    hit: Option<String>,
    crit: Option<String>,
}

impl HitSounds {
    pub fn sound(&self, kind: HitKind) -> Option<&str> {
        use HitKind::*;
        match kind {
            Miss => self.miss.as_deref(),
            Graze => self.graze.as_deref(),
            Hit => self.hit.as_deref(),
            Crit => self.crit.as_deref(),
            Auto => self.hit.as_deref(),
        }
    }
}

impl Default for HitSounds {
    fn default() -> Self {
        HitSounds { miss: None, graze: None, hit: None, crit: None }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum AttackKindBuilder {
    Melee { reach: f32 },
    Ranged { range: f32, projectile: String },
}
