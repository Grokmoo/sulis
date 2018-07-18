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

use sulis_core::util::{ExtInt, Point};
use sulis_rules::BonusList;
use script::ScriptCallback;
use ChangeListenerList;

use ROUND_TIME_MILLIS;

struct Surface {
    area_id: String,
    points: Vec<Point>,
}

pub struct Effect {
    pub name: String,
    pub tag: String,
    cur_duration: u32,
    total_duration: ExtInt,

    bonuses: BonusList,
    callbacks: Vec<Rc<ScriptCallback>>,

    pub listeners: ChangeListenerList<Effect>,
    pub removal_listeners: ChangeListenerList<Effect>,

    deactivate_with_ability: Option<String>,
    surface: Option<Surface>,
}

impl Effect {
    pub fn new(name: &str, tag: &str, duration: ExtInt, bonuses: BonusList,
               deactivate_with_ability: Option<String>) -> Effect {
        Effect {
            name: name.to_string(),
            tag: tag.to_string(),
            cur_duration: 0,
            total_duration: duration,
            bonuses,
            listeners: ChangeListenerList::default(),
            removal_listeners: ChangeListenerList::default(),
            callbacks: Vec::new(),
            deactivate_with_ability,
            surface: None
        }
    }

    pub fn mark_for_removal(&mut self) {
        self.total_duration = ExtInt::Int(0);
    }

    pub fn set_surface_for_area(&mut self, area: &str, points: &Vec<Point>) {
        let area_id = area.to_string();
        self.surface = Some(Surface {
            area_id,
            points: points.clone(),
        })
    }

    pub fn surface(&self) -> Option<(&str, &Vec<Point>)> {
        match self.surface {
            None => None,
            Some(ref surface) => Some((&surface.area_id, &surface.points)),
        }
    }

    pub fn deactivates_with(&self, id: &str) -> bool {
        match self.deactivate_with_ability {
            None => false,
            Some(ref ability_id) => id == ability_id,
        }
    }

    pub fn callbacks(&self) -> Vec<Rc<ScriptCallback>> {
        self.callbacks.clone()
    }

    pub fn add_callback(&mut self, cb: Rc<ScriptCallback>) {
        self.callbacks.push(cb);
    }

    /// Updates the effect time.  returns true if a round has elapsed
    #[must_use]
    pub fn update(&mut self, millis_elapsed: u32) -> Vec<Rc<ScriptCallback>> {
        let cur_mod = self.cur_duration / ROUND_TIME_MILLIS;

        self.cur_duration += millis_elapsed;

        if cur_mod != self.cur_duration / ROUND_TIME_MILLIS {
            self.listeners.notify(&self);

            return self.callbacks.clone()
        }

        Vec::new()
    }

    pub fn is_removal(&self) -> bool {
        match self.total_duration {
            ExtInt::Infinity => false,
            ExtInt::Int(total_duration) => {
                if self.cur_duration >= total_duration {
                    self.removal_listeners.notify(&self);
                    debug!("Removing effect");
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bonuses(&self) -> &BonusList {
        &self.bonuses
    }

    pub fn duration_millis(&self) -> ExtInt {
        self.total_duration
    }

    pub fn total_duration_rounds(&self) -> ExtInt {
        match self.total_duration {
            ExtInt::Infinity => ExtInt::Infinity,
            ExtInt::Int(dur) => {
                ExtInt::Int((dur as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32)
            }
        }
    }

    pub fn remaining_duration_rounds(&self) -> ExtInt {
        let remaining_duration = self.total_duration - self.cur_duration;
        match remaining_duration {
            ExtInt::Infinity => ExtInt::Infinity,
            ExtInt::Int(dur) => {
                ExtInt::Int((dur as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32)
            }
        }
    }
}
