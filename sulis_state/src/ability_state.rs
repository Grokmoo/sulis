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
use std::u32;

use crate::{ChangeListenerList, GameState};
use sulis_core::util::ExtInt;
use sulis_module::{ability::Duration, Ability, Module, StatList, ROUND_TIME_MILLIS};

pub struct AbilityState {
    pub ability: Rc<Ability>,
    pub group: String,
    pub(crate) remaining_duration: ExtInt,
    pub combat_only: bool,
    pub requires_melee: bool,
    pub requires_ranged: bool,
    pub requires_active_mode: Option<Rc<Ability>>,
    cur_duration: u32,
    pub listeners: ChangeListenerList<AbilityState>,
    pub newly_added_ability: bool,
}

impl AbilityState {
    pub fn new(ability: &Rc<Ability>) -> AbilityState {
        let (group, combat_only, mode, melee, ranged) = match ability.active {
            None => panic!(),
            Some(ref active) => {
                let mode = active
                    .requires_active_mode
                    .as_ref()
                    .map_or(None, |id| Module::ability(&id));
                if mode.is_none() && active.requires_active_mode.is_some() {
                    warn!("Invalid requires_active_mode for {}", ability.id);
                }
                (active.group.name(), active.combat_only, mode,
                 active.requires_melee, active.requires_ranged)
            }
        };

        AbilityState {
            ability: Rc::clone(ability),
            group,
            remaining_duration: ExtInt::Int(0),
            combat_only,
            cur_duration: 0,
            requires_active_mode: mode,
            requires_melee: melee,
            requires_ranged: ranged,
            listeners: ChangeListenerList::default(),
            newly_added_ability: false,
        }
    }

    pub fn update(&mut self, millis_elapsed: u32) {
        let cur_mod = self.cur_duration / ROUND_TIME_MILLIS;
        self.cur_duration += millis_elapsed;

        self.remaining_duration = self.remaining_duration - millis_elapsed;

        if cur_mod != self.cur_duration / ROUND_TIME_MILLIS {
            self.listeners.notify(&self);
        }
    }

    pub fn activate_ap(&self) -> u32 {
        self.ability.active.as_ref().unwrap().ap
    }

    pub fn is_available(&self, stats: &StatList, current_modes: &[&str]) -> bool {
        if self.requires_melee && !stats.attack_is_melee() { return false; }
        if self.requires_ranged && !stats.attack_is_ranged() { return false; }

        if let Some(required_mode) = self.requires_active_mode.as_ref() {
            if !current_modes.contains(&&required_mode.id[..]) {
                return false;
            }
        }

        if self.combat_only && !GameState::is_combat_active() {
            return false;
        }
        self.remaining_duration.is_zero()
    }

    pub fn is_active_mode(&self) -> bool {
        self.remaining_duration.is_infinite()
    }

    pub fn activate(&mut self) {
        self.remaining_duration = match self.ability.active {
            None => panic!(),
            Some(ref active) => match active.duration {
                Duration::Mode => ExtInt::Infinity,
                Duration::Permanent | Duration::Instant | Duration::Rounds(_) => {
                    ExtInt::Int(active.cooldown * ROUND_TIME_MILLIS)
                }
            },
        };
        self.cur_duration = 0;
        self.listeners.notify(&self);
    }

    pub fn set_cooldown_rounds(&mut self, rounds: u32) {
        self.remaining_duration = ExtInt::Int(rounds * ROUND_TIME_MILLIS);
        self.cur_duration = 0;
        self.listeners.notify(&self);
    }

    pub fn deactivate(&mut self) {
        if !self.is_active_mode() {
            warn!(
                "Attempted to deactivate non-mode ability {}",
                self.ability.id
            );
            return;
        }

        self.remaining_duration = match self.ability.active {
            None => panic!(),
            Some(ref active) => ExtInt::Int(active.cooldown * ROUND_TIME_MILLIS),
        };
        self.listeners.notify(&self);
    }

    pub fn remaining_duration(&self) -> ExtInt {
        self.remaining_duration
    }

    pub fn remaining_duration_rounds(&self) -> ExtInt {
        match self.remaining_duration {
            ExtInt::Infinity => ExtInt::Infinity,
            ExtInt::Int(dur) => ExtInt::Int((dur as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32),
        }
    }
}
