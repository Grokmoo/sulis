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

mod path_finder;
pub use self::path_finder::{Destination, LocationChecker, PathFinder};

mod path_finder_grid;
pub use self::path_finder_grid::PathFinderGrid;

pub mod tile;
pub use self::tile::Tile;
pub use self::tile::Tileset;

use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serializer};

use sulis_core::image::Image;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::util::{unable_to_create_error, Point, Size};

use crate::generator::{EncounterParams, EncounterParamsBuilder, PropParams, PropParamsBuilder};
use crate::{Encounter, ItemListEntrySaveState, Module, ObjectSize, OnTrigger, Prop};

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
    pub image_display: Rc<dyn Image>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActorData {
    pub id: String,
    pub location: Point,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<String>,
}

#[derive(Clone)]
pub struct PropData {
    pub prop: Rc<Prop>,
    pub location: Point,
    pub items: Vec<ItemListEntrySaveState>,
    pub enabled: bool,
    pub hover_text: Option<String>,
}

#[derive(Clone)]
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
    pub location_kind: LocationKind,
    pub generator: Option<GeneratorParams>,
    pub builder: AreaBuilder,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(mut builder: AreaBuilder) -> Result<Area, Error> {
        let mut props = Vec::new();
        for prop_builder in builder.props.iter() {
            let prop_data = create_prop(prop_builder)?;
            props.push(prop_data);
        }

        let mut transitions: Vec<Transition> = Vec::new();
        for (index, t_builder) in builder.transitions.iter().enumerate() {
            let image = match ResourceSet::image(&t_builder.image_display) {
                None => {
                    warn!(
                        "Image '{}' not found for transition.",
                        t_builder.image_display
                    );
                    continue;
                }
                Some(image) => image,
            };

            let size = match Module::size(&t_builder.size) {
                None => {
                    warn!("Size '{}' not found for transition.", t_builder.size);
                    continue;
                }
                Some(ref size) => Rc::clone(size),
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

            debug!(
                "Created transition to '{:?}' at {},{}",
                t_builder.to, t_builder.from.x, t_builder.from.y
            );

            let transition = Transition {
                from: t_builder.from,
                to: t_builder.to.clone(),
                hover_text: t_builder.hover_text.clone(),
                size,
                image_display: image,
            };
            transitions.push(transition);
        }

        let mut triggers: Vec<Trigger> = Vec::new();
        for tbuilder in &builder.triggers {
            triggers.push(Trigger {
                kind: tbuilder.kind.clone(),
                on_activate: tbuilder.on_activate.clone(),
                initially_enabled: tbuilder.initially_enabled,
            });
        }

        let mut used_triggers = HashSet::new();
        let mut encounters = Vec::new();
        for encounter_builder in builder.encounters.iter() {
            let encounter = match Module::encounter(&encounter_builder.id) {
                None => {
                    warn!("No encounter '{}' found", &encounter_builder.id);
                    return unable_to_create_error("area", &builder.id);
                }
                Some(encounter) => encounter,
            };

            let mut encounter_triggers = Vec::new();
            for (index, trigger) in triggers.iter().enumerate() {
                match trigger.kind {
                    TriggerKind::OnEncounterCleared { encounter_location }
                    | TriggerKind::OnEncounterActivated { encounter_location } => {
                        if encounter_location == encounter_builder.location {
                            encounter_triggers.push(index);
                            used_triggers.insert(index);
                        }
                    }
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
                TriggerKind::OnEncounterCleared { encounter_location }
                | TriggerKind::OnEncounterActivated { encounter_location } => {
                    if !used_triggers.contains(&index) {
                        warn!(
                            "Invalid encounter trigger at point {:?}",
                            encounter_location
                        );
                    }
                }
                _ => (),
            }
        }

        let visibility_tile = ResourceSet::sprite(&builder.visibility_tile)?;
        let explored_tile = ResourceSet::sprite(&builder.explored_tile)?;

        let generator = match builder.generator.take() {
            None => None,
            Some(gen) => Some(GeneratorParams::new(gen)?),
        };

        Ok(Area {
            id: builder.id.to_string(),
            name: builder.name.to_string(),
            width: builder.width as i32,
            height: builder.height as i32,
            actors: builder.actors.clone(),
            encounters,
            props,
            visibility_tile,
            explored_tile,
            transitions,
            triggers,
            vis_dist: builder.max_vis_distance,
            vis_dist_squared: builder.max_vis_distance * builder.max_vis_distance,
            vis_dist_up_one_squared: builder.max_vis_up_one_distance
                * builder.max_vis_up_one_distance,
            world_map_location: builder.world_map_location.clone(),
            on_rest: builder.on_rest.clone(),
            location_kind: builder.location_kind.clone(),
            generator,
            builder,
        })
    }

    pub fn coords_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        if x >= self.width || y >= self.height {
            return false;
        }

        true
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AreaBuilder {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub visibility_tile: String,
    pub explored_tile: String,
    pub max_vis_distance: i32,
    pub max_vis_up_one_distance: i32,
    pub world_map_location: Option<String>,
    pub on_rest: OnRest,
    pub location_kind: LocationKind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<GeneratorParamsBuilder>,
    pub layers: Vec<String>,
    pub entity_layer: usize,
    pub actors: Vec<ActorData>,
    pub props: Vec<PropDataBuilder>,
    pub encounters: Vec<EncounterDataBuilder>,
    pub transitions: Vec<TransitionBuilder>,
    pub triggers: Vec<TriggerBuilder>,

    #[serde(serialize_with = "ser_terrain", deserialize_with = "de_terrain")]
    pub terrain: Vec<Option<String>>,

    #[serde(serialize_with = "ser_walls", deserialize_with = "de_walls")]
    pub walls: Vec<(u8, Option<String>)>,

    #[serde(serialize_with = "ser_layer_set", deserialize_with = "de_layer_set")]
    pub layer_set: HashMap<String, Vec<Vec<u16>>>,

    #[serde(serialize_with = "as_base64", deserialize_with = "from_base64")]
    pub elevation: Vec<u8>,
}

pub struct GeneratorParams {
    pub id: String,

    pub transitions: Vec<TransitionAreaParams>,
    pub encounters: EncounterParams,
    pub props: PropParams,
}

impl GeneratorParams {
    pub fn new(builder: GeneratorParamsBuilder) -> Result<GeneratorParams, Error> {
        Ok(GeneratorParams {
            id: builder.id,
            transitions: builder.transitions,
            encounters: EncounterParams::new(builder.encounters)?,
            props: PropParams::new(builder.props)?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct GeneratorParamsBuilder {
    id: String,

    #[serde(default)]
    pub transitions: Vec<TransitionAreaParams>,

    #[serde(default)]
    pub encounters: EncounterParamsBuilder,

    #[serde(default)]
    pub props: PropParamsBuilder,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TransitionAreaParams {
    pub to: String,
    pub kind: String,
    pub hover_text: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
struct U8WithKinds {
    kinds: Vec<String>,
    entries: String,
}

fn entry_index<'a>(
    map: &mut HashMap<&'a str, u8>,
    index: &mut u8,
    entry: &'a Option<String>,
) -> Result<u8, Error> {
    Ok(match entry {
        None => 255,
        Some(ref id) => {
            let index = map.entry(id).or_insert_with(|| {
                let ret_val = *index;
                *index += 1;
                ret_val
            });

            if *index > 254 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Can only serialize up to 255 wall kinds",
                ));
            }

            *index
        }
    })
}

fn serialize_u8_with_kinds<'a, S>(
    kinds: HashMap<&str, u8>,
    name: &'static str,
    vec: Vec<u8>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut kinds: Vec<_> = kinds.into_iter().collect();
    kinds.sort_by_key(|k| k.1);
    let kinds = kinds.into_iter().map(|k| k.0).collect::<Vec<&str>>();

    let mut data = serializer.serialize_struct(name, 2)?;
    data.serialize_field("kinds", &kinds)?;
    data.serialize_field("entries", &base64::encode(&vec))?;
    data.end()
}

fn de_terrain<'de, D>(deserializer: D) -> Result<Vec<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let input = U8WithKinds::deserialize(deserializer)?;
    use sulis_core::serde::de::Error;
    let vec_u8 = base64::decode(&input.entries).map_err(|err| Error::custom(err.to_string()))?;

    let mut out = Vec::new();
    for entry in vec_u8 {
        let index = entry as usize;
        if index == 255 {
            out.push(None);
        } else if index >= input.kinds.len() {
            return Err(Error::custom("Invalid base64 encoding in terrain index."));
        } else {
            out.push(Some(input.kinds[index].clone()));
        }
    }

    Ok(out)
}

fn ser_terrain<S>(input: &Vec<Option<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut kinds: HashMap<&str, u8> = HashMap::new();
    let mut terrain: Vec<u8> = Vec::new();

    let mut index = 0;
    for terrain_id in input.iter() {
        use sulis_core::serde::ser::Error;
        let entry_index = entry_index(&mut kinds, &mut index, terrain_id)
            .map_err(|e| Error::custom(e.to_string()))?;

        terrain.push(entry_index as u8);
    }

    serialize_u8_with_kinds(kinds, "terrain", terrain, serializer)
}

fn de_walls<'de, D>(deserializer: D) -> Result<Vec<(u8, Option<String>)>, D::Error>
where
    D: Deserializer<'de>,
{
    let input = U8WithKinds::deserialize(deserializer)?;
    use sulis_core::serde::de::Error;
    let vec_u8 = base64::decode(&input.entries).map_err(|err| Error::custom(err.to_string()))?;

    let mut out = Vec::new();
    if vec_u8.is_empty() {
        return Ok(out);
    }

    let mut i = 0;
    loop {
        if i + 2 > vec_u8.len() {
            return Err(Error::custom("Invalid base64 encoding in walls"));
        }

        let elev = vec_u8[i + 1];
        let index = vec_u8[i] as usize;

        if index == 255 {
            out.push((elev, None));
        } else if index >= input.kinds.len() {
            return Err(Error::custom("Invalid base64 encoding in walls index"));
        } else {
            out.push((elev, Some(input.kinds[index].clone())));
        }

        i += 2;
        if i == vec_u8.len() {
            break;
        }
    }

    Ok(out)
}

fn ser_walls<S>(input: &Vec<(u8, Option<String>)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut kinds: HashMap<&str, u8> = HashMap::new();
    let mut walls: Vec<u8> = Vec::new();

    let mut index = 0;
    for (level, wall_id) in input.iter() {
        use sulis_core::serde::ser::Error;
        let entry_index = entry_index(&mut kinds, &mut index, wall_id)
            .map_err(|e| Error::custom(e.to_string()))?;

        walls.push(entry_index as u8);
        walls.push(*level);
    }

    serialize_u8_with_kinds(kinds, "walls", walls, serializer)
}

fn ser_layer_set<S>(
    input: &HashMap<String, Vec<Vec<u16>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(input.len()))?;
    for (key, vec) in input.iter() {
        let mut out: Vec<u8> = Vec::new();
        for pos in vec.iter() {
            out.push(((pos[0] >> 8) & 0xff) as u8);
            out.push((pos[0] & 0xff) as u8);
            out.push(((pos[1] >> 8) & 0xff) as u8);
            out.push((pos[1] & 0xff) as u8);
        }
        map.serialize_entry(key, &base64::encode(&out))?;
    }

    map.end()
}

fn de_layer_set<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<Vec<u16>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let input: HashMap<String, String> = HashMap::deserialize(deserializer)?;

    let mut result: HashMap<String, Vec<Vec<u16>>> = HashMap::new();
    for (key, encoded) in input {
        use sulis_core::serde::de::Error;
        let vec_u8 = base64::decode(&encoded).map_err(|err| Error::custom(err.to_string()))?;

        let mut result_vec: Vec<Vec<u16>> = Vec::new();
        let mut i = 0;
        if vec_u8.is_empty() {
            continue;
        }
        loop {
            if i + 4 > vec_u8.len() {
                return Err(Error::custom("Invalid encoded base64 string"));
            }
            let x = vec_u8[i] as u16 * 256 + vec_u8[i + 1] as u16;
            let y = vec_u8[i + 2] as u16 * 256 + vec_u8[i + 3] as u16;
            result_vec.push(vec![x, y]);

            if i + 4 == vec_u8.len() {
                break;
            }

            i += 4;
        }
        result.insert(key, result_vec);
    }

    Ok(result)
}

fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use sulis_core::serde::de::Error;
    let s = String::deserialize(deserializer)?;
    base64::decode(&s).map_err(|err| Error::custom(err.to_string()))
}

fn as_base64<S>(input: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(input))
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum LocationKind {
    Outdoors,
    Indoors,
    Underground,
}

impl LocationKind {
    pub fn iter() -> impl Iterator<Item = &'static LocationKind> {
        use crate::area::LocationKind::*;
        [Outdoors, Indoors, Underground].iter()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum OnRest {
    Disabled { message: String },
    FireScript { id: String, func: String },
}

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
    Area {
        id: String,
        x: i32,
        y: i32,
    },
    CurArea {
        x: i32,
        y: i32,
    },
    WorldMap,
    FindLink {
        id: String,
        x_offset: i32,
        y_offset: i32,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemListEntrySaveState>,
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_text: Option<String>,
}

pub fn create_prop(builder: &PropDataBuilder) -> Result<PropData, Error> {
    let prop = match Module::prop(&builder.id) {
        None => return unable_to_create_error("prop", &builder.id),
        Some(prop) => prop,
    };

    let location = builder.location;

    let enabled = builder.enabled.unwrap_or(true);

    Ok(PropData {
        prop,
        location,
        items: builder.items.clone(),
        enabled,
        hover_text: builder.hover_text.clone(),
    })
}
