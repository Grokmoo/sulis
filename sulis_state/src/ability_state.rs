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

use std::u32;
use std::rc::Rc;

use sulis_module::{ability::Duration, Ability};
use ChangeListenerList;

use ROUND_TIME_MILLIS;

pub struct AbilityState {
    pub ability: Rc<Ability>,
    pub group: String,
    pub(crate) remaining_duration: u32,
    pub listeners: ChangeListenerList<AbilityState>,
}

impl AbilityState {
    pub fn new(ability: &Rc<Ability>) -> AbilityState {
        let group = match ability.active {
            None => panic!(),
            Some(ref active) => active.group.name(),
        };

        AbilityState {
            ability: Rc::clone(ability),
            group,
            remaining_duration: 0,
            listeners: ChangeListenerList::default(),
        }
    }

    pub fn update(&mut self, millis_elapsed: u32) {
        let cur_rounds = self.remaining_duration % ROUND_TIME_MILLIS;

        if millis_elapsed > self.remaining_duration {
            self.remaining_duration = 0;
        } else {
            self.remaining_duration -= millis_elapsed;
        }

        if cur_rounds != self.remaining_duration % ROUND_TIME_MILLIS {
            self.listeners.notify(&self);
        }
    }

    pub fn activate_ap(&self) -> u32 {
        self.ability.active.as_ref().unwrap().ap
    }

    pub fn is_available(&self) -> bool {
        self.remaining_duration == 0
    }

    pub fn is_active_mode(&self) -> bool {
        self.remaining_duration > 100_000 * ROUND_TIME_MILLIS
    }

    pub fn activate(&mut self) {
        self.remaining_duration = match self.ability.active {
            None => panic!(),
            Some(ref active) => match active.duration {
                Duration::Mode => u32::MAX,
                Duration::Instant | Duration::Rounds(_) => active.cooldown * ROUND_TIME_MILLIS,
            }
        };
    }

    pub fn deactivate(&mut self) {
        if !self.is_active_mode() {
            warn!("Attempted to deactivate non-mode ability {}", self.ability.id);
            return;
        }

        self.remaining_duration = match self.ability.active {
            None => panic!(),
            Some(ref active) => active.cooldown * ROUND_TIME_MILLIS,
        };
        self.listeners.notify(&self);
    }

    pub fn remaining_duration(&self) -> u32 {
        self.remaining_duration
    }

    pub fn remaining_duration_rounds(&self) -> u32 {
        (self.remaining_duration as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32
    }
}
