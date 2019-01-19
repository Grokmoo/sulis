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
use std::io::Error;
use std::cmp::max;

use sulis_core::ui::{color, Color};
use sulis_core::util::{gen_rand, invalid_data_error};
use sulis_rules::{DamageList, Armor, DamageKind, Resistance, Time, ROUND_TIME_MILLIS};
use crate::area::LocationKind;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub id: String,
    pub base_ap: u32,
    pub max_overflow_ap: i32,
    pub min_overflow_ap: i32,
    pub max_ap: u32,
    pub movement_ap: u32,
    pub attack_ap: u32,
    pub display_ap: u32,
    pub swap_weapons_ap: u32,
    pub initiative_roll_max: i32,
    pub base_flanking_angle: i32,
    pub graze_percentile: u32,
    pub hit_percentile: u32,
    pub crit_chance: u32,

    pub flanking_accuracy_bonus: i32,
    pub hidden_accuracy_bonus: i32,

    pub graze_damage_multiplier: f32,
    pub crit_damage_multiplier: f32,

    pub dual_wield_damage_multiplier: f32,

    pub base_attribute: i32,
    pub builder_max_attribute: i32,
    pub builder_min_attribute: i32,
    pub builder_attribute_points: i32,

    pub max_dialog_distance: f32,
    pub max_transition_distance: f32,
    pub max_prop_distance: f32,

    pub selectable_races: Vec<String>,
    pub selectable_classes: Vec<String>,
    pub ability_groups: Vec<String>,

    pub experience_for_level: Vec<u32>,

    pub loot_drop_prop: String,

    pub item_weight_display_factor: f32,
    pub item_value_display_factor: f32,

    pub coins_item: String,

    armor_damage_reduction_cap: Vec<u32>,

    pub rounds_per_hour: u32,
    pub hours_per_day: u32,
    pub hour_names: Vec<String>,

    pub area_colors: HashMap<LocationKind, Vec<Color>>,
}

impl Rules {
    pub fn canonicalize_time(&self, time: &mut Time) {
        let mut millis = time.millis;
        let mut round = time.round;
        let mut hour = time.hour;
        let mut day = time.day;

        if millis >= ROUND_TIME_MILLIS {
            round += millis / ROUND_TIME_MILLIS;
            millis = millis % ROUND_TIME_MILLIS;
        }

        if round >= self.rounds_per_hour {
            hour += round / self.rounds_per_hour;
            round = round % self.rounds_per_hour;
        }

        if hour >= self.hours_per_day {
            day += hour / self.hours_per_day;
            hour = hour % self.hours_per_day;
        }

        time.millis = millis;
        time.round = round;
        time.hour = hour;
        time.day = day;
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.hour_names.len() != self.hours_per_day as usize {
            return invalid_data_error(&format!("Must specify '{}' hours names to match number of hours",
                                              self.hours_per_day));
        }

        for (_, colors) in self.area_colors.iter() {
            if colors.len() != self.hours_per_day as usize {
                return invalid_data_error(&format!("Must specify '{}' hours for each area_colors.",
                                                   self.hours_per_day));
            }
        }

        Ok(())
    }

    pub fn compute_millis(&self, time: Time) -> usize {
        let mut millis = time.millis as usize;

        let mut factor = ROUND_TIME_MILLIS as usize;
        millis += time.round as usize * factor;

        factor *= self.rounds_per_hour as usize;
        millis += time.hour as usize * factor;

        factor *= self.hours_per_day as usize;
        millis += time.day as usize * factor;

        millis
    }

    pub fn get_hour_name(&self, hour: u32) -> &str {
        assert!((hour as usize) < self.hour_names.len());

        &self.hour_names[hour as usize]
    }

    pub fn get_area_color(&self, location_kind: LocationKind, time: Time) -> Color {
        let (prev, next) = match self.area_colors.get(&location_kind) {
            None => return color::WHITE,
            Some(colors) => {
                let prev = time.hour as usize;
                if prev == colors.len() - 1 {
                    (colors[prev], colors[0])
                } else {
                    (colors[prev], colors[prev + 1])
                }
            }
        };

        let round_frac = 1.0 / self.rounds_per_hour as f32;
        let next_frac = round_frac * (time.round as f32 + time.millis as f32 / ROUND_TIME_MILLIS as f32);
        let prev_frac = 1.0 - next_frac;

        let r = prev_frac * prev.r + next_frac * next.r;
        let g = prev_frac * prev.g + next_frac * next.g;
        let b = prev_frac * prev.b + next_frac * next.b;
        let a = prev_frac * prev.a + next_frac * next.a;
        Color { r, g, b, a }
    }

    /// Computes the amount of damage that this damage list will apply to the given
    /// `armor`.  Each damage component of this list is rolled randomly, with the resulting
    /// damage then multiplied by the `multiplier`, rounded down.  The damage is then
    /// modified by the percentage resistance, if any.  The armor against
    /// the base damage kind of this damage is then subtracted from the damage, capped
    /// by the armor damage reduction cap for that armor value.  The
    /// resulting vector may be an empty vector to indicate no damage, or a vector of
    /// one or more kinds each associated with a positive damage amount.  The damage
    /// amount for each entry will never be zero.
    pub fn roll_damage(&self, damage: &DamageList, armor: &Armor, resistance: &Resistance,
                       multiplier: f32) -> Vec<(DamageKind, u32)> {
        debug!("Rolling damage from {} to {} vs {} base armor",
               damage.min(), damage.max(), armor.base());

        if damage.is_empty() { return Vec::new(); }

        let mut output = Vec::new();
        for damage in damage.iter() {
            let kind = damage.kind.unwrap();

            let resistance = (100 - resistance.amount(kind)) as f32 / 100.0;
            let amount = damage.roll() as f32 * multiplier * resistance;

            let armor = max(0, armor.amount(kind) as i32 - damage.ap as i32) as u32;
            let armor_max = self.armor_damage_reduction_cap(armor) as f32 * amount / 100.0;
            let armor = armor as f32;

            let armor = if armor_max > armor { armor } else { armor_max };
            let armor = if armor > amount { amount } else { armor };

            let amount = amount - armor;
            if amount > 0.0 {
                output.push((kind, amount.ceil() as u32));
            }
        }

        output
    }

    /// Returns the percentile armor reduction cap for the given armor value.  this
    /// is the maximum percentage that the armor of that level can reduce a damage
    /// amount by.  the remaining damage is rounded up.
    pub fn armor_damage_reduction_cap(&self, armor: u32) -> u32 {
        *self.armor_damage_reduction_cap.get(armor as usize).unwrap_or(&100)
    }

    pub fn get_xp_for_next_level(&self, cur_level: u32) -> u32 {
        if cur_level < 1 { return 0; }
        if cur_level - 1 >= self.experience_for_level.len() as u32 { return 0; }

        self.experience_for_level[(cur_level - 1) as usize]
    }

    pub fn concealment_roll(&self, concealment: i32) -> bool {
        if concealment == 0 { return true; }
        let roll = gen_rand(1, 101);
        debug!("Concealment roll: {} against {}", roll, concealment);
        roll > concealment
    }
}
