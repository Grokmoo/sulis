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

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::script::{script_callback::FuncKind, CallbackData};
use crate::{save_state::EffectSaveState, ChangeListenerList, EntityState};
use sulis_core::util::{invalid_data_error, ExtInt, Point};
use sulis_module::{BonusList, ROUND_TIME_MILLIS};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub(crate) area_id: String,
    pub(crate) points: Vec<Point>,
    pub(crate) squares_to_fire_on_moved: u32,

    #[serde(default)]
    pub(crate) aura: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    pub icon: String,
    pub text: String,
}

pub struct Effect {
    pub name: String,
    pub tag: String,

    pub ui_visible: bool,

    pub(crate) cur_duration: u32,
    pub(crate) total_duration: ExtInt,
    pub(crate) bonuses: BonusList,
    pub(crate) deactivate_with_ability: Option<String>,
    pub(crate) surface: Option<Surface>,
    pub(crate) entity: Option<usize>,
    pub(crate) callbacks: Vec<Rc<CallbackData>>,
    pub(crate) icon: Option<Icon>,

    squares_moved: HashMap<usize, u32>,

    pub listeners: ChangeListenerList<Effect>,

    // listeners that are called when this effect is removed
    // child animations are referenced here to cause them to
    // be removed when this effect is removed
    pub removal_listeners: ChangeListenerList<Effect>,
}

impl Effect {
    pub fn load(
        data: EffectSaveState,
        new_index: usize,
        entities: &HashMap<usize, Rc<RefCell<EntityState>>>,
    ) -> Result<Effect, Error> {
        let mut callbacks = Vec::new();
        for mut cb in data.callbacks {
            cb.update_entity_refs_on_load(entities)?;
            cb.update_effect_index_on_load(new_index);
            callbacks.push(Rc::new(cb));
        }

        let mut surface = data.surface;
        if let Some(ref mut surface) = surface {
            if let Some(aura) = surface.aura.take() {
                match entities.get(&aura) {
                    None => {
                        return invalid_data_error(&format!(
                            "Invalid aura parent {aura} for effect"
                        ));
                    }
                    Some(entity) => {
                        surface.aura = Some(entity.borrow().index());
                    }
                }
            }
        }

        Ok(Effect {
            name: data.name,
            tag: data.tag,
            ui_visible: data.ui_visible,
            cur_duration: data.cur_duration,
            total_duration: data.total_duration,
            bonuses: data.bonuses,
            deactivate_with_ability: data.deactivate_with_ability,
            surface,
            entity: data.entity,
            icon: data.icon,

            squares_moved: HashMap::new(),
            callbacks,
            listeners: ChangeListenerList::default(),
            removal_listeners: ChangeListenerList::default(),
        })
    }

    pub fn new(
        name: &str,
        tag: &str,
        duration: ExtInt,
        bonuses: BonusList,
        deactivate_with_ability: Option<String>,
    ) -> Effect {
        Effect {
            name: name.to_string(),
            tag: tag.to_string(),
            ui_visible: true,
            cur_duration: 0,
            total_duration: duration,
            bonuses,
            listeners: ChangeListenerList::default(),
            removal_listeners: ChangeListenerList::default(),
            callbacks: Vec::new(),
            deactivate_with_ability,
            surface: None,
            entity: None,
            icon: None,
            squares_moved: HashMap::new(),
        }
    }

    pub(crate) fn increment_squares_moved(&mut self, entity: usize) {
        *self.squares_moved.entry(entity).or_insert(0) += 1;
    }

    pub fn mark_for_removal(&mut self) {
        self.total_duration = ExtInt::Int(0);

        // prevent any more scripts from firing except on_removed
        for cb in self.callbacks.iter_mut() {
            // DANGER
            // there will be at least one other Rc of this callback outstanding
            // frequently - from the script that is resulting in a call to this
            // func.  in that case, using make_mut to clone on write should be fine.
            let cb = Rc::make_mut(cb);
            cb.clear_funcs_except(&[FuncKind::OnRemoved, FuncKind::OnExitedSurface]);
        }
    }

    pub fn set_surface_for_area(
        &mut self,
        area: &str,
        points: &[Point],
        squares_to_fire_on_moved: u32,
        aura: Option<usize>,
    ) {
        let area_id = area.to_string();
        self.surface = Some(Surface {
            area_id,
            points: points.to_vec(),
            squares_to_fire_on_moved,
            aura,
        })
    }

    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    pub fn set_icon(&mut self, icon: String, text: String) {
        self.icon = Some(Icon { icon, text });
    }

    pub fn set_owning_entity(&mut self, entity: usize) {
        self.entity = Some(entity);
    }

    pub fn is_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub fn is_aura(&self) -> bool {
        match &self.surface {
            None => false,
            Some(surface) => surface.aura.is_some(),
        }
    }

    pub fn surface(&self) -> Option<(&str, &Vec<Point>)> {
        self.surface
            .as_ref()
            .map(|s| (s.area_id.as_str(), &s.points))
    }

    pub fn deactivates_with(&self, id: &str) -> bool {
        match self.deactivate_with_ability {
            None => false,
            Some(ref ability_id) => id == ability_id,
        }
    }

    pub fn callbacks(&self) -> Vec<Rc<CallbackData>> {
        self.callbacks.clone()
    }

    pub fn add_callback(&mut self, cb: Rc<CallbackData>) {
        self.callbacks.push(cb);
    }

    #[must_use]
    pub fn update_on_moved_in_surface(&mut self) -> Vec<(Rc<CallbackData>, usize)> {
        let to_fire = match self.surface {
            None => return Vec::new(),
            Some(ref surface) => surface.squares_to_fire_on_moved,
        };

        let mut result = Vec::new();
        for (index, amount) in self.squares_moved.iter_mut() {
            if *amount >= to_fire {
                for cb in self.callbacks.iter() {
                    result.push((cb.clone(), *index));
                }

                *amount -= to_fire;
            }
        }

        result
    }

    /// Updates the effect time.  returns true if a round has elapsed
    #[must_use]
    pub fn update(&mut self, millis_elapsed: u32) -> Vec<Rc<CallbackData>> {
        let cur_mod = self.cur_duration / ROUND_TIME_MILLIS;

        self.cur_duration += millis_elapsed;

        if cur_mod != self.cur_duration / ROUND_TIME_MILLIS {
            self.listeners.notify(self);

            return self.callbacks.clone();
        }

        Vec::new()
    }

    pub fn is_removal(&self) -> bool {
        match self.total_duration {
            ExtInt::Infinity => false,
            ExtInt::Int(total_duration) => {
                if self.cur_duration >= total_duration {
                    self.removal_listeners.notify(self);
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
            ExtInt::Int(dur) => ExtInt::Int((dur as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32),
        }
    }

    pub fn remaining_duration_rounds(&self) -> ExtInt {
        let remaining_duration = self.total_duration - self.cur_duration;
        match remaining_duration {
            ExtInt::Infinity => ExtInt::Infinity,
            ExtInt::Int(dur) => ExtInt::Int((dur as f32 / ROUND_TIME_MILLIS as f32).ceil() as u32),
        }
    }
}
