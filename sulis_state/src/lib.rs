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

extern crate chrono;
extern crate rlua;
extern crate rand;
extern crate int_hash;

extern crate sulis_core;
extern crate sulis_module;
extern crate sulis_rules;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod ai;
pub use self::ai::AI;

mod ability_state;
pub use self::ability_state::AbilityState;

mod actor_state;
pub use self::actor_state::ActorState;

pub mod animation;

pub mod area_feedback_text;
use self::area_feedback_text::AreaFeedbackText;

mod area_state;
pub use self::area_state::AreaState;

mod change_listener;
pub use self::change_listener::ChangeListener;
pub use self::change_listener::ChangeListenerList;

mod effect;
pub use self::effect::Effect;

mod entity_state;
pub use self::entity_state::EntityState;
pub use self::entity_state::AreaDrawable;

mod entity_texture_cache;
pub use self::entity_texture_cache::EntityTextureCache;
pub use self::entity_texture_cache::EntityTextureSlot;

mod formation;
pub use self::formation::Formation;

mod game_state;
pub use self::game_state::GameState;

mod item_state;
pub use self::item_state::ItemState;

pub mod inventory;
pub use self::inventory::Inventory;

pub mod item_list;
pub use self::item_list::ItemList;

mod location;
pub use self::location::Location;

mod los_calculator;
pub use self::los_calculator::calculate_los;
pub use self::los_calculator::has_visibility;

mod merchant;
pub use self::merchant::Merchant;

mod path_finder;
use self::path_finder::PathFinder;

mod party_stash;
pub use self::party_stash::PartyStash;

mod prop_state;
pub use self::prop_state::PropState;

pub mod quest_state;
pub use self::quest_state::QuestState;
pub use self::quest_state::QuestStateSet;

pub mod save_file;
pub use self::save_file::SaveFile;
pub use self::save_file::SaveFileMetaData;

mod save_state;
pub use self::save_state::SaveState;

pub mod script;
pub use self::script::ScriptState;
pub use self::script::ScriptCallback;

mod turn_manager;
pub(crate) use self::turn_manager::TurnManager;
pub use self::turn_manager::ROUND_TIME_MILLIS;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_module::{Actor, OnTrigger, Module};

pub const MOVE_TO_THRESHOLD: f32 = 0.4;

#[derive(Debug)]
pub enum NextGameStep {
    Exit,
    NewCampaign { pc_actor: Rc<Actor> },
    LoadCampaign { save_state: SaveState },
    LoadModuleAndNewCampaign { pc_actor: Rc<Actor>, module_dir: String },
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
            locations.insert(location.id.clone(), WorldMapLocationState {
                visible: location.initially_visible,
                enabled: location.initially_enabled,
            });
        }

        WorldMapState { locations }
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
