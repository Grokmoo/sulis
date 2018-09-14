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

use std::slice::Iter;
use std::fmt::{self, Display};
use rand::{self, Rng};

use Armor;

#[derive(Clone)]
pub struct DamageList {
    damage: Vec<Damage>,
    min: u32,
    max: u32,
    ap: u32,
}

impl DamageList {
    pub fn to_vec(self) -> Vec<Damage> {
        self.damage
    }

    pub fn empty() -> DamageList {
        DamageList {
            damage: Vec::new(),
            min: 0,
            max: 0,
            ap: 0,
        }
    }

    pub fn from(damage: Damage) -> DamageList {
        if damage.kind.is_none() {
            warn!("Attempted to create damage list with no base damage kind");
            return DamageList::empty();
        }

        let min = damage.min;
        let max = damage.max;
        let ap = damage.ap;

        DamageList {
            damage: vec![damage],
            min,
            max,
            ap,
        }
    }

    pub fn new(base_damage: Damage, bonus_damage: &Vec<Damage>) -> DamageList {
        if base_damage.kind.is_none() {
            warn!("Attempted to create damage list with no base damage kind");
            return DamageList::empty();
        }

        let mut damage_list = Vec::new();
        let mut min = base_damage.min;
        let mut max = base_damage.max;
        let mut ap = base_damage.ap;
        damage_list.push(base_damage);

        let mut bonus_damage = bonus_damage.clone();
        bonus_damage.sort_by_key(|d| d.kind);

        let mut cur_damage = None;
        for damage in bonus_damage {
            min += damage.min;
            max += damage.max;
            ap += damage.ap;

            if damage.kind.is_none() || damage.kind == damage_list[0].kind {
                damage_list[0].add(damage);
                continue;
            }

            match cur_damage {
                None => {
                    cur_damage = Some(damage);
                }, Some(mut cur_damage_unwrapped) => {
                    if cur_damage_unwrapped.kind == damage.kind {
                        cur_damage_unwrapped.add(damage);
                    } else {
                        assert!(cur_damage_unwrapped.kind.is_some());
                        damage_list.push(cur_damage_unwrapped);
                        cur_damage = Some(damage);
                    }
                }
            }
        }

        if let Some(cur_damage) = cur_damage {
            assert!(cur_damage.kind.is_some());
    damage_list.push(cur_damage);
        }

        debug!("Created damage list {} to {}, base kind {}", min,
               max, damage_list[0].kind.unwrap());
        for damage in damage_list.iter() {
            trace!("Component: {} to {}, kind {:?}", damage.min, damage.max, damage.kind);
        }

        DamageList {
            damage: damage_list,
            min,
            max,
            ap,
        }
    }

    /// Computes the amount of damage that this damage list will apply to the given
    /// `armor`.  Each damage component of this list is rolled randomly, with the resulting
    /// damage then multiplied by the `multiplier`, rounded down.  The armor against
    /// the base damage kind of this damage is then subtracted from the damage.  The
    /// resulting vector may be an empty vector to indicate no damage, or a vector of
    /// one or more kinds each associated with a positive damage amount.  The damage
    /// amount for each entry will never be zero.
    pub fn roll(&self, armor: &Armor, multiplier: f32) -> Vec<(DamageKind, u32)> {
        if self.damage.is_empty() { return Vec::new(); }

        let armor_amount = armor.amount(self.damage[0].kind.unwrap());

        debug!("Computing damage amount from {} to {} vs {} armor", self.min,
               self.max, armor_amount);

        let armor_amount = if self.ap > armor_amount {
            0
        } else {
            armor_amount - self.ap
        };
        trace!("Armor adjusted by AP: {}", armor_amount);

        let mut output = Vec::new();
        let mut armor_left = armor_amount;
        for damage in self.damage.iter() {
            let mut damage_amount = (damage.roll() as f32 * multiplier) as u32;
            let kind = damage.kind.unwrap();

            if armor_left >= damage_amount {
                armor_left -= damage_amount;
            } else {
                damage_amount -= armor_left;
                armor_left = 0;
                output.push((kind, damage_amount));
            }
        }

        output
    }

    pub fn ap(&self) -> u32 { self.ap }

    pub fn min(&self) -> u32 { self.min }

    pub fn max(&self) -> u32 { self.max }

    pub fn mult(&self, multiplier: f32) -> DamageList {
        let mut damage_vec = Vec::new();
        let mut min = 0;
        let mut max = 0;
        let mut ap = 0;
        for damage in self.damage.iter() {
            let new_damage = damage.clone().mult_f32(multiplier);
            min += new_damage.min;
            max += new_damage.max;
            ap += new_damage.ap;
            damage_vec.push(new_damage);
        }

        DamageList {
            damage: damage_vec,
            min,
            max,
            ap,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum DamageKind {
    Slashing,
    Piercing,
    Crushing,
    Acid,
    Cold,
    Electrical,
    Fire,
    Sonic,
    Raw,
}

use DamageKind::*;

// This array MUST be in the same order as the ordering on the DamageKind enum
// This is top to bottom declaration order for derived.
const DAMAGE_KINDS: [DamageKind; 9] = [Slashing, Piercing, Crushing, Acid, Cold,
    Electrical, Fire, Sonic, Raw];

impl DamageKind {
    pub fn iter() -> Iter<'static, DamageKind> {
        DAMAGE_KINDS.iter()
    }

    pub fn index(&self) -> usize {
        use DamageKind::*;
        match self {
            &Slashing => 0,
            &Piercing => 1,
            &Crushing => 2,
            &Acid => 3,
            &Cold => 4,
            &Electrical => 5,
            &Fire => 6,
            &Sonic => 7,
            &Raw => 8,
        }
    }

    pub fn from_str(s: &str) -> DamageKind {
        match s {
            "Slashing" => DamageKind::Slashing,
            "Piercing" => DamageKind::Piercing,
            "Crushing" => DamageKind::Crushing,
            "Acid" => DamageKind::Acid,
            "Cold" => DamageKind::Cold,
            "Electrical" => DamageKind::Electrical,
            "Fire" => DamageKind::Fire,
            "Sonic" => DamageKind::Sonic,
            "Raw" => DamageKind::Raw,
            _ => {
                warn!("Unable to parse '{}' as damage kind", s);
                DamageKind::Raw
            }
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            &Slashing => "Slashing",
            &Piercing => "Piercing",
            &Crushing => "Crushing",
            &Acid => "Acid",
            &Cold => "Cold",
            &Electrical => "Electrical",
            &Fire => "Fire",
            &Sonic => "Sonic",
            &Raw => "Raw",
        }
    }
}

impl Display for DamageKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Damage {
    // note that if this is ever changed to allow negative values, bonus.apply_modifiers
    // needs to be updated to account for that case
    pub min: u32,
    pub max: u32,

    #[serde(default)]
    pub ap: u32,
    pub kind: Option<DamageKind>,
}

impl Damage {
    pub fn add(&mut self, other: Damage) {
        self.min += other.min;
        self.max += other.max;
        self.ap += other.ap;
    }

    pub fn mult_f32_mut(&mut self, val: f32) {
        self.min = (self.min as f32 * val) as u32;
        self.max = (self.max as f32 * val) as u32;
        self.ap = (self.ap as f32 * val) as u32;
    }

    pub fn mult_f32(&self, val: f32) -> Damage {
        Damage {
            min: (self.min as f32 * val) as u32,
            max: (self.max as f32 * val) as u32,
            ap: (self.ap as f32 * val) as u32,
            kind: self.kind,
        }
    }

    pub fn mult(&self, times: u32) -> Damage {
        Damage {
            min: self.min * times,
            max: self.max * times,
            ap: self.ap * times,
            kind: self.kind,
        }
    }

    pub fn average(&self) -> f32 {
        (self.min as f32 + self.max as f32) / 2.0
    }

    pub fn roll(&self) -> u32 {
        rand::thread_rng().gen_range(self.min, self.max + 1)
    }
}

impl Default for Damage {
    fn default() -> Damage {
        Damage {
            min: 0,
            max: 0,
            ap: 0,
            kind: None,
        }
    }
}
