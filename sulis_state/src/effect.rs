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

use std::time::Instant;

use sulis_core::util;
use sulis_rules::BonusList;

pub struct Effect {
    start_time: Instant,
    duration: u32,

    bonuses: BonusList,
}

impl Effect {
    pub fn new(duration: u32, bonuses: BonusList) -> Effect {
        Effect {
            start_time: Instant::now(),
            duration,
            bonuses,
        }
    }

    pub fn update(&self) -> bool {
        let millis = util::get_elapsed_millis(self.start_time.elapsed());

        if millis < self.duration {
            true
        } else {
            debug!("Removing effect");
            false
        }
    }

    pub fn bonuses(&self) -> &BonusList {
        &self.bonuses
    }

    pub fn duration_millis(&self) -> u32 {
        self.duration
    }
}
