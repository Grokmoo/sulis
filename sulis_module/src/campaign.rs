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
use std::rc::Rc;
use std::io::Error;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use sulis_core::util::{unable_to_create_error, Point};
use sulis_rules::Time;

use crate::{Conversation, Module, on_trigger};

pub struct WorldMap {
    pub size: (f32, f32),
    pub offset: (f32, f32),
    pub locations: Vec<WorldMapLocation>,
}

pub struct WorldMapLocation {
    pub id: String,
    pub name: String,
    pub position: (f32, f32),
    pub icon: Rc<Image>,
    pub initially_enabled: bool,
    pub initially_visible: bool,

    pub linked_area: Option<String>,
    pub linked_area_pos: Point,

    pub travel_times: HashMap<String, u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CampaignGroup {
    pub id: String,
    pub name: String,
    pub position: usize,
}

impl Hash for CampaignGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for CampaignGroup {
    fn eq(&self, other: &CampaignGroup) -> bool {
        self.id == other.id
    }
}

impl Eq for CampaignGroup {}

impl Ord for CampaignGroup {
    fn cmp(&self, other: &CampaignGroup) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for CampaignGroup {
    fn partial_cmp(&self, other: &CampaignGroup) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Campaign {
    pub id: String,
    pub starting_time: Time,
    pub starting_area: String,
    pub starting_location: Point,
    pub name: String,
    pub description: String,
    pub backstory_conversation: Rc<Conversation>,
    pub max_starting_level: u32,
    pub on_party_death_script: on_trigger::ScriptData,
    pub on_tick_script: Option<on_trigger::ScriptData>,
    pub on_round_elapsed_script: Option<on_trigger::ScriptData>,
    pub world_map: WorldMap,
    pub group: Option<CampaignGroup>,
}

impl Campaign {
    pub fn new(builder: CampaignBuilder) -> Result<Campaign, Error> {

        let backstory_conversation = match Module::conversation(&builder.backstory_conversation) {
            None => {
                warn!("Backstory conversation '{}' not found", &builder.backstory_conversation);
                return unable_to_create_error("module", &builder.name);
            }, Some(convo) => convo,
        };

        let mut locations = Vec::new();
        for (id, location) in builder.world_map.locations {
            let image = match ResourceSet::image(&location.icon) {
                None => {
                    warn!("Invalid image for '{}': '{}'", id, location.icon);
                    return unable_to_create_error("module", &builder.name);
                }, Some(img) => img,
            };

            locations.push(WorldMapLocation {
                id,
                name: location.name,
                icon: image,
                position: location.position,
                initially_enabled: location.initially_enabled,
                initially_visible: location.initially_visible,
                linked_area: location.linked_area,
                linked_area_pos: location.linked_area_pos,
                travel_times: location.travel_times,
            });
        }

        Ok(Campaign {
            group: builder.group,
            starting_time: builder.starting_time,
            starting_area: builder.starting_area,
            starting_location: builder.starting_location,
            name: builder.name,
            description: builder.description,
            backstory_conversation,
            id: builder.id,
            max_starting_level: builder.max_starting_level,
            on_party_death_script: builder.on_party_death_script,
            on_tick_script: builder.on_tick_script,
            on_round_elapsed_script: builder.on_round_elapsed_script,
            world_map: WorldMap {
                size: builder.world_map.size,
                offset: builder.world_map.offset,
                locations,
            }
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CampaignBuilder {
    pub id: String,
    pub group: Option<CampaignGroup>,
    pub starting_time: Time,
    pub starting_area: String,
    pub starting_location: Point,
    pub name: String,
    pub description: String,
    pub backstory_conversation: String,
    pub max_starting_level: u32,
    pub on_party_death_script: on_trigger::ScriptData,
    pub on_tick_script: Option<on_trigger::ScriptData>,
    pub on_round_elapsed_script: Option<on_trigger::ScriptData>,
    pub world_map: WorldMapBuilder,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WorldMapLocationBuilder {
    pub name: String,
    pub position: (f32, f32),
    pub icon: String,

    #[serde(default="bool_true")]
    pub initially_enabled: bool,
    #[serde(default="bool_true")]
    pub initially_visible: bool,

    pub linked_area: Option<String>,
    #[serde(default)]
    pub linked_area_pos: Point,

    #[serde(default)]
    pub travel_times: HashMap<String, u32>,
}

fn bool_true() -> bool { true }

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WorldMapBuilder {
    pub size: (f32, f32),
    pub offset: (f32, f32),
    pub locations: HashMap<String, WorldMapLocationBuilder>,
}
