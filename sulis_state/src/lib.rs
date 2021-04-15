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

#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod ai;
pub use self::ai::AI;

pub mod ability_state;
pub use self::ability_state::AbilityState;

mod actor_state;
pub use self::actor_state::ActorState;

pub mod animation;

pub mod area_feedback_text;
pub use self::area_feedback_text::AreaFeedbackText;

pub mod area_state;
pub use self::area_state::AreaState;

mod change_listener;
pub use self::change_listener::ChangeListener;
pub use self::change_listener::ChangeListenerList;

mod distance_finder;
pub use self::distance_finder::{
    can_attack, center, center_i32, dist, is_threat, is_within, is_within_attack_dist,
    is_within_touch_dist, Locatable,
};

mod effect;
pub use self::effect::Effect;

mod entity_attack_handler;

mod entity_state;
pub use self::entity_state::AreaDrawable;
pub use self::entity_state::EntityState;

mod entity_texture_cache;
pub use self::entity_texture_cache::EntityTextureCache;
pub use self::entity_texture_cache::EntityTextureSlot;

mod formation;
pub use self::formation::Formation;

mod game_state;
pub use self::game_state::GameState;

mod generated_area;
pub use self::generated_area::{GeneratedArea, PregenOutput};

pub mod inventory;
pub use self::inventory::Inventory;

pub mod item_list;
pub use self::item_list::ItemList;

mod location;
pub use self::location::Location;

mod los_calculator;
pub use self::los_calculator::calculate_los;
pub use self::los_calculator::has_visibility;

mod merchant_state;
pub use self::merchant_state::MerchantState;

mod path_finder;

mod party_bump_handler;

mod party_stash;
pub use self::party_stash::PartyStash;

mod prop_state;
pub use self::prop_state::PropState;

mod p_stats;
pub use self::p_stats::PStats;

pub mod quest_state;
pub use self::quest_state::QuestState;
pub use self::quest_state::QuestStateSet;

mod range_indicator;
pub use self::range_indicator::{RangeIndicator, RangeIndicatorHandler, RangeIndicatorImageSet};

pub mod save_file;
pub use self::save_file::SaveFile;
pub use self::save_file::SaveFileMetaData;

mod save_state;
pub use self::save_state::SaveState;

pub mod script;
pub use self::script::{Script, ScriptCallback, ScriptState};

mod transition_handler;

mod turn_manager;
pub(crate) use self::turn_manager::TurnManager;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use sulis_module::{Actor, Module, OnTrigger};

#[derive(Debug)]
pub enum NextGameStep {
    Exit,
    NewCampaign {
        pc_actor: Rc<Actor>,
    },
    LoadCampaign {
        save_state: Box<SaveState>,
    },
    LoadModuleAndNewCampaign {
        pc_actor: Rc<Actor>,
        party_actors: Vec<Rc<Actor>>,
        flags: HashMap<String, String>,
        module_dir: String,
    },
    MainMenu,
    MainMenuReloadResources,
    RecreateIO,
}

pub struct UICallback {
    pub on_trigger: Vec<OnTrigger>,
    pub parent: Rc<RefCell<EntityState>>,
    pub target: Rc<RefCell<EntityState>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldMapState {
    locations: HashMap<String, WorldMapLocationState>,
}

impl WorldMapState {
    fn new() -> WorldMapState {
        let campaign = Module::campaign();
        let map = &campaign.world_map;

        let mut locations = HashMap::new();
        for location in map.locations.iter() {
            locations.insert(
                location.id.clone(),
                WorldMapLocationState {
                    visible: location.initially_visible,
                    enabled: location.initially_enabled,
                },
            );
        }

        WorldMapState { locations }
    }

    fn load(&mut self) {
        let campaign = Module::campaign();
        let map = &campaign.world_map;

        for location in map.locations.iter() {
            if self.locations.contains_key(&location.id) {
                continue;
            }

            self.locations.insert(
                location.id.clone(),
                WorldMapLocationState {
                    visible: location.initially_visible,
                    enabled: location.initially_enabled,
                },
            );
        }
    }

    pub fn is_visible(&self, location: &str) -> bool {
        if let Some(ref state) = self.locations.get(location) {
            state.visible
        } else {
            warn!("Location '{}' not found when querying visible", location);
            false
        }
    }

    pub fn is_enabled(&self, location: &str) -> bool {
        if let Some(ref state) = self.locations.get(location) {
            state.enabled
        } else {
            warn!("Location '{}' not found when querying enabled", location);
            false
        }
    }

    fn set_visible(&mut self, location: &str, visible: bool) {
        if let Some(ref mut state) = self.locations.get_mut(location) {
            state.visible = visible;
        } else {
            warn!("Location '{}' not found when setting visible", location);
        }
    }

    fn set_enabled(&mut self, location: &str, enabled: bool) {
        if let Some(ref mut state) = self.locations.get_mut(location) {
            state.enabled = enabled;
        } else {
            warn!("Location '{}' not found when setting enabled", location);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WorldMapLocationState {
    pub visible: bool,
    pub enabled: bool,
}
