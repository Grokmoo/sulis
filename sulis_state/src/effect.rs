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

use sulis_rules::BonusList;

use ROUND_TIME_MILLIS;

pub struct Effect {
    name: String,
    cur_duration: u32,
    total_duration: u32,

    bonuses: BonusList,
}

impl Effect {
    pub fn new(name: &str, duration: u32, bonuses: BonusList) -> Effect {
        Effect {
            name: name.to_string(),
            cur_duration: 0,
            total_duration: duration,
            bonuses,
        }
    }

    pub fn update(&mut self, millis_elapsed: u32) {
        self.cur_duration += millis_elapsed;
    }

    pub fn is_removal(&self) -> bool {
        if self.cur_duration < self.total_duration {
            false
        } else {
            debug!("Removing effect");
            true
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bonuses(&self) -> &BonusList {
        &self.bonuses
    }

    pub fn duration_millis(&self) -> u32 {
        self.total_duration
    }

    pub fn total_duration_rounds(&self) -> u32 {
        ((self.total_duration / ROUND_TIME_MILLIS) as f32).ceil() as u32
    }

    pub fn remaining_duration_rounds(&self) -> u32 {
        (((self.total_duration - self.cur_duration) / ROUND_TIME_MILLIS) as f32).ceil() as u32
    }
}
