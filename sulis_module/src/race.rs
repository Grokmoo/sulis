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

use std::io::Error;
use std::rc::Rc;
use std::collections::HashMap;

use sulis_core::image::Image;
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::util::{invalid_data_error, unable_to_create_error, Point};
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_rules::BonusList;

use actor::{Sex};

use {EntitySize, ImageLayer, ImageLayerSet, Module};

pub struct Race {
    pub id: String,
    pub name: String,
    pub size: Rc<EntitySize>,
    pub player: bool,
    pub base_stats: BonusList,
    pub hair_selections: Vec<String>,
    pub beard_selections: Vec<String>,

    default_images: ImageLayerSet,
    image_layer_offsets: HashMap<ImageLayer, (f32, f32)>,
    image_layer_postfix: HashMap<Sex, String>,
}

impl PartialEq for Race {
    fn eq(&self, other: &Race) -> bool {
        self.id == other.id
    }
}

impl Race {
    pub fn new(builder: RaceBuilder, module: &Module) -> Result<Race, Error> {
        let size = match module.sizes.get(&builder.size) {
            None => {
                warn!("No match found for size '{}'", builder.size);
                return unable_to_create_error("race", &builder.id);
            }, Some(size) => Rc::clone(size)
        };

        let mut offsets = HashMap::new();
        let scale = builder.image_layer_offset_scale as f32;
        for (layer, p) in builder.image_layer_offsets {
            offsets.insert(layer, (p.x as f32 / scale, p.y as f32 / scale));
        }

        let default_images = ImageLayerSet::new(builder.default_images)?;

        match builder.base_stats.attack {
            None => {
                warn!("You must specify a base_damage for each race.");
                return unable_to_create_error("race", &builder.id);
            }, Some(ref attack) => {
                if attack.damage.kind.is_none() {
                    warn!("Attack must always have a damage kind specified.");
                    return unable_to_create_error("race", &builder.id);
                }
            }
        }

        Ok(Race {
            id: builder.id,
            name: builder.name,
            size,
            player: builder.player.unwrap_or(false),
            base_stats: builder.base_stats,
            default_images,
            image_layer_offsets: offsets,
            image_layer_postfix: builder.image_layer_postfix,
            hair_selections: builder.hair_selections.unwrap_or(Vec::new()),
            beard_selections: builder.beard_selections.unwrap_or(Vec::new()),
        })
    }

    pub fn image_for_sex(&self, sex: Sex, image: &Rc<Image>) -> Rc<Image> {
        match self.image_layer_postfix.get(&sex) {
            None => Rc::clone(image),
            Some(ref postfix) => {
                let id = image.id() + postfix;
                match ResourceSet::get_image(&id) {
                    None => Rc::clone(image),
                    Some(ref new_image) => Rc::clone(new_image),
                }
            }
        }
    }

    pub fn get_image_layer_offset(&self, layer: ImageLayer) -> (f32, f32) {
        *self.image_layer_offsets.get(&layer).unwrap_or(&(0.0, 0.0))
    }

    pub fn default_images(&self) -> &ImageLayerSet {
        &self.default_images
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RaceBuilder {
    pub id: String,
    pub name: String,
    pub size: usize,
    pub player: Option<bool>,
    pub base_stats: BonusList,
    pub default_images: HashMap<Sex, HashMap<ImageLayer, String>>,
    pub hair_selections: Option<Vec<String>>,
    pub beard_selections: Option<Vec<String>>,
    image_layer_offsets: HashMap<ImageLayer, Point>,
    image_layer_offset_scale: i32,
    image_layer_postfix: HashMap<Sex, String>,
}

impl ResourceBuilder for RaceBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<RaceBuilder, Error> {
        let resource: RaceBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<RaceBuilder, Error> {
        let resource: Result<RaceBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
