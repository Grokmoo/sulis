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

use std::rc::Rc;
use std::cell::RefCell;

use sulis_module::{Actor, OnTrigger};

pub const MOVE_TO_THRESHOLD: f32 = 0.4;

#[derive(Debug)]
pub enum NextGameStep {
    Exit,
    NewCampaign { pc_actor: Rc<Actor> },
    LoadCampaign { save_state: SaveState },
    MainMenu,
    RecreateIO,
}

pub struct UICallback {
    pub on_trigger: OnTrigger,
    pub parent: Rc<RefCell<EntityState>>,
    pub target: Rc<RefCell<EntityState>>,
}
