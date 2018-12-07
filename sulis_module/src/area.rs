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

mod layer;
pub use self::layer::Layer;

mod layer_set;
pub use self::layer_set::LayerSet;

mod path_finder_grid;
use self::path_finder_grid::PathFinderGrid;

pub mod tile;
pub use self::tile::Tile;
pub use self::tile::Tileset;

use std::collections::{HashSet, HashMap};
use std::io::{Error};
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::util::{Point, Size, unable_to_create_error};

use crate::{Encounter, Module, ObjectSize, OnTrigger, Prop, ItemListEntrySaveState};

pub const MAX_AREA_SIZE: i32 = 128;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TriggerKind {
    OnCampaignStart,
    OnAreaLoad,
    OnPlayerEnter { location: Point, size: Size },
    OnEncounterCleared { encounter_location: Point },
    OnEncounterActivated { encounter_location: Point },
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub kind: TriggerKind,
    pub on_activate: Vec<OnTrigger>,
    pub initially_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: Point,
    pub size: Rc<ObjectSize>,
    pub to: ToKind,
    pub hover_text: String,
    pub image_display: Rc<Image>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActorData {
    pub id: String,
    pub location: Point,
    pub unique_id: Option<String>,
}

pub struct PropData {
    pub prop: Rc<Prop>,
    pub location: Point,
    pub items: Vec<ItemListEntrySaveState>,
    pub enabled: bool,
}

pub struct EncounterData {
    pub encounter: Rc<Encounter>,
    pub location: Point,
    pub size: Size,
    pub triggers: Vec<usize>,
}

pub struct Area {
    pub id: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub layer_set: LayerSet,
    path_grids: HashMap<String, PathFinderGrid>,
    pub visibility_tile: Rc<Sprite>,
    pub explored_tile: Rc<Sprite>,
    pub actors: Vec<ActorData>,
    pub props: Vec<PropData>,
    pub transitions: Vec<Transition>,
    pub encounters: Vec<EncounterData>,
    pub triggers: Vec<Trigger>,
    pub vis_dist: i32,
    pub vis_dist_squared: i32,
    pub vis_dist_up_one_squared: i32,
    pub world_map_location: Option<String>,
    pub on_rest: OnRest,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(builder: AreaBuilder, module: &Module) -> Result<Area, Error> {
        let mut props = Vec::new();
        for prop_builder in builder.props.iter() {
            let prop_data = create_prop(prop_builder, module)?;
            props.push(prop_data);
        }

        info!("Creating area {}", builder.id);
        let layer_set = LayerSet::new(&builder, module, &props);
        let layer_set = match layer_set {
            Ok(l) => l,
            Err(e) => {
                warn!("Unable to generate layer_set for area '{}'", builder.id);
                return Err(e);
            }
        };

        let mut path_grids = HashMap::new();
        for (_, size) in module.sizes.iter() {
            let path_grid = PathFinderGrid::new(Rc::clone(size), &layer_set);
            debug!("Generated path grid for size {}", size.id);
            path_grids.insert(size.id.to_string(), path_grid);
        }

        // TODO validate position of all actors, props, encounters

        let mut transitions: Vec<Transition> = Vec::new();
        for (index, t_builder) in builder.transitions.into_iter().enumerate() {
            let image = match ResourceSet::get_image(&t_builder.image_display) {
                None => {
                    warn!("Image '{}' not found for transition.", t_builder.image_display);
                    continue;
                },
                Some(image) => image,
            };

            let size = match module.sizes.get(&t_builder.size) {
                None => {
                    warn!("Size '{}' not found for transition.", t_builder.size);
                    continue;
                }, Some(ref size) => Rc::clone(size),
            };

            let p = t_builder.from;
            if !p.in_bounds(builder.width as i32, builder.height as i32) {
                warn!("Transition {} falls outside area bounds", index);
                continue;
            }
            p.add(size.width, size.height);
            if !p.in_bounds(builder.width as i32, builder.height as i32) {
                warn!("Transition {} falls outside area bounds", index);
                continue;
            }

            debug!("Created transition to '{:?}' at {},{}", t_builder.to,
                   t_builder.from.x, t_builder.from.y);

            let transition = Transition {
                from: t_builder.from,
                to: t_builder.to,
                hover_text: t_builder.hover_text,
                size,
                image_display: image,
            };
            transitions.push(transition);
        }

        let mut triggers: Vec<Trigger> = Vec::new();
        for tbuilder in builder.triggers {
            triggers.push(Trigger {
                kind: tbuilder.kind,
                on_activate: tbuilder.on_activate,
                initially_enabled: tbuilder.initially_enabled,
            });
        }

        let mut used_triggers = HashSet::new();
        let mut encounters = Vec::new();
        for encounter_builder in builder.encounters {
            let encounter = match module.encounters.get(&encounter_builder.id) {
                None => {
                    warn!("No encounter '{}' found", &encounter_builder.id);
                    return unable_to_create_error("area", &builder.id);
                }, Some(encounter) => Rc::clone(encounter),
            };

            let mut encounter_triggers = Vec::new();
            for (index, trigger) in triggers.iter().enumerate() {
                match trigger.kind {
                    TriggerKind::OnEncounterCleared { encounter_location } |
                        TriggerKind::OnEncounterActivated { encounter_location } => {
                        if encounter_location == encounter_builder.location {
                            encounter_triggers.push(index);
                            used_triggers.insert(index);
                        }
                    },
                    _ => (),
                }
            }

            encounters.push(EncounterData {
                encounter,
                location: encounter_builder.location,
                size: encounter_builder.size,
                triggers: encounter_triggers,
            });
        }

        for (index, trigger) in triggers.iter().enumerate() {
            match trigger.kind {
                TriggerKind::OnEncounterCleared { encounter_location } |
                    TriggerKind::OnEncounterActivated { encounter_location } => {
                    if !used_triggers.contains(&index) {
                        warn!("Invalid encounter trigger at point {:?}", encounter_location);
                    }
                },
                _ => (),
            }
        }

        let visibility_tile = ResourceSet::get_sprite(&builder.visibility_tile)?;
        let explored_tile = ResourceSet::get_sprite(&builder.explored_tile)?;

        Ok(Area {
            id: builder.id,
            name: builder.name,
            width: builder.width as i32,
            height: builder.height as i32,
            layer_set: layer_set,
            path_grids: path_grids,
            actors: builder.actors,
            encounters,
            props,
            visibility_tile,
            explored_tile,
            transitions,
            triggers,
            vis_dist: builder.max_vis_distance,
            vis_dist_squared: builder.max_vis_distance * builder.max_vis_distance,
            vis_dist_up_one_squared: builder.max_vis_up_one_distance * builder.max_vis_up_one_distance,
            world_map_location: builder.world_map_location,
            on_rest: builder.on_rest,
        })
    }

    pub fn coords_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 { return false; }
        if x >= self.width || y >= self.height { return false; }

        true
    }

    pub fn get_path_grid(&self, size_id: &str) -> &PathFinderGrid {
        self.path_grids.get(size_id).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AreaBuilder {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub generate: bool,
    pub layers: Vec<String>,
    pub entity_layer: usize,
    pub actors: Vec<ActorData>,
    pub props: Vec<PropDataBuilder>,
    pub encounters: Vec<EncounterDataBuilder>,
    pub transitions: Vec<TransitionBuilder>,
    pub triggers: Vec<TriggerBuilder>,
    pub visibility_tile: String,
    pub explored_tile: String,
    pub max_vis_distance: i32,
    pub max_vis_up_one_distance: i32,
    pub world_map_location: Option<String>,
    pub on_rest: OnRest,
    pub layer_set: HashMap<String, Vec<Vec<usize>>>,
    pub terrain: Vec<Option<String>>,
    pub walls: Vec<(i8, Option<String>)>,
    // #[serde(serialize_with="ser_elevation", deserialize_with="from_base64")]
    pub elevation: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum OnRest {
    Disabled { message: String },
    FireScript { id: String, func: String },
}

// fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error> where D: Deserializer<'de> {
//     use sulis_core::serde::de::Error;
//     let s = String::deserialize(deserializer)?;
//     base64::decode(&s).map_err(|err| Error::custom(err.to_string()))
// }
//
// fn as_base64<S>(input: &[u8], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//     serializer.serialize_str(&base64::encode(input))
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TriggerBuilder {
    pub kind: TriggerKind,
    pub on_activate: Vec<OnTrigger>,
    pub initially_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum ToKind {
    Area { id: String, x: i32, y: i32 },
    CurArea { x: i32, y: i32 },
    WorldMap,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransitionBuilder {
    pub from: Point,
    pub size: String,
    pub to: ToKind,
    pub hover_text: String,
    pub image_display: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EncounterDataBuilder {
    pub id: String,
    pub location: Point,
    pub size: Size,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PropDataBuilder {
    pub id: String,
    pub location: Point,
    #[serde(default)]
    pub items: Vec<ItemListEntrySaveState>,
    pub enabled: Option<bool>,
}

pub fn create_prop(builder: &PropDataBuilder, module: &Module) -> Result<PropData, Error> {
    let prop = match module.props.get(&builder.id) {
        None => return unable_to_create_error("prop", &builder.id),
        Some(prop) => Rc::clone(prop),
    };

    let location = builder.location;

    let enabled = builder.enabled.unwrap_or(true);

    Ok(PropData {
        prop,
        location,
        items: builder.items.clone(),
        enabled,
    })
}
