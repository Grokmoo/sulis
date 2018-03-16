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

use sulis_core::image::Image;
use {Armor, AttributeList, Attack, BonusList, Damage};
use bonus_list::AttackBuilder;

#[derive(Clone)]
pub struct StatList {
    attack_range: f32,

    pub attributes: AttributeList,
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
}

impl StatList {
    pub fn new(attrs: AttributeList) -> StatList {
        StatList {
            attributes: attrs,
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

    /// Adds the bonuses from the specified BonusList to this stat list.
    pub fn add(&mut self, bonuses: &BonusList) {
        self.add_multiple(bonuses, 1);
    }

    /// Adds the specified bonuses to this StatList the specified number of times.
    /// Note that non-numeric bonuses are only added once regardless of the value of
    /// times
    pub fn add_multiple(&mut self, bonuses: &BonusList, times: u32) {
        if times == 0 { return; }

        self.armor.add(bonuses.base_armor, &bonuses.armor_kinds);

        if let Some(bonus_damage) = bonuses.bonus_damage {
            self.bonus_damage.push(bonus_damage.mult(times));
        }

        let times_f32 = times as f32;
        let times_i32 = times as i32;
        if let Some(reach) = bonuses.bonus_reach { self.bonus_reach += reach * times_f32; }
        if let Some(range) = bonuses.bonus_range { self.bonus_range += range * times_f32; }
        if let Some(hit_points) = bonuses.hit_points { self.max_hp += hit_points * times_i32; }
        if let Some(initiative) = bonuses.initiative { self.initiative += initiative * times_i32; }
        if let Some(accuracy) = bonuses.accuracy { self.accuracy += accuracy * times_i32; }
        if let Some(defense) = bonuses.defense { self.defense += defense * times_i32; }
        if let Some(fortitude) = bonuses.fortitude { self.fortitude += fortitude * times_i32; }
        if let Some(reflex) = bonuses.reflex { self.reflex += reflex * times_i32; }
        if let Some(will) = bonuses.will { self.will += will * times_i32; }

        if let Some(ref attrs) = bonuses.attributes {
            self.attributes.add_all(attrs);
        }
    }

    pub fn finalize(&mut self, attacks: Vec<&AttackBuilder>, multiplier: f32, base_attr: i32) {
        if attacks.is_empty() {
            warn!("Finalized stats with no attacks");
        }

        let mut attack_range = None;
        for builder in attacks {
            let attack = Attack::new(builder, &self).mult(multiplier);

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
    }
}

