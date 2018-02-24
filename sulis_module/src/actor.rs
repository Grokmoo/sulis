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

use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

use sulis_core::io::GraphicsRenderer;
use sulis_core::image::{Image, LayeredImage};
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::ui::{Color};
use sulis_core::util::invalid_data_error;
use sulis_core::{serde_json, serde_yaml};
use sulis_rules::AttributeList;

use {Class, ImageLayer, ImageLayerSet, Item, Module, Race};

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Sex {
    Male,
    Female,
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Actor {
    pub id: String,
    pub name: String,
    pub player: bool,
    pub portrait: Option<Rc<Image>>,
    pub race: Rc<Race>,
    pub sex: Sex,
    pub attributes: AttributeList,
    pub items: Vec<Rc<Item>>,
    pub to_equip: Vec<usize>,
    pub levels: Vec<(Rc<Class>, u32)>,
    pub hue: Option<f32>,
    pub hair_color: Option<Color>,
    pub skin_color: Option<Color>,
    image_layers: ImageLayerSet,
    image: LayeredImage,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn new(builder: ActorBuilder, resources: &Module) -> Result<Actor, Error> {
        let race = match resources.races.get(&builder.race) {
            None => {
                warn!("No match found for race '{}'", builder.race);
                return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
            }, Some(race) => Rc::clone(race)
        };

        let mut items: Vec<Rc<Item>> = Vec::new();
        if let Some(builder_items) = builder.items {
            for item_id in builder_items {
                let item = match resources.items.get(&item_id) {
                    None => {
                        warn!("No match found for item ID '{}'", item_id);
                        return Err(Error::new(ErrorKind::InvalidData,
                             format!("Unable to create actor '{}'", builder.id)));
                    },
                    Some(item) => Rc::clone(item)
                };
                items.push(item);
            }
        }

        let sex = match builder.sex {
            None => Sex::Male,
            Some(sex) => sex,
        };

        let mut levels: Vec<(Rc<Class>, u32)> = Vec::new();
        for (class_id, level) in builder.levels {
            let class = match resources.classes.get(&class_id) {
                None => {
                    warn!("No match for class '{}'", class_id);
                    return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
                }, Some(class) => Rc::clone(class)
            };

            levels.push((class, level));
        }

        let mut to_equip: Vec<usize> = Vec::new();
        if let Some(builder_equipped) = builder.equipped {
            for index in builder_equipped {
                let index = index as usize;
                if index >= items.len() {
                    warn!("No item exists to equip at index {}", index);
                    return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
                }

                if items[index].equippable.is_none() {
                    warn!("Item at index {}: {} is not equippable.", index, items[index].name);
                    return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
                }

                to_equip.push(index);
            }
        }

        let portrait = match builder.portrait {
            None => None,
            Some(ref image) => match ResourceSet::get_image(image) {
                None => {
                    warn!("Unable to find image for portrait '{}'", image);
                    None
                }, Some(image) => Some(image),
            }
        };

        let image_layers = ImageLayerSet::merge(race.default_images(), sex, builder.images)?;
        let images_list = image_layers.get_list(sex, builder.hair_color, builder.skin_color);
        let image = LayeredImage::new(images_list, builder.hue);

        Ok(Actor {
            id: builder.id,
            name: builder.name,
            player: builder.player.unwrap_or(false),
            portrait,
            race,
            sex,
            attributes: builder.attributes,
            levels,
            items,
            to_equip,
            image_layers,
            image,
            hue: builder.hue,
            skin_color: builder.skin_color,
            hair_color: builder.hair_color,
        })
    }

    pub fn image_layers(&self) -> &ImageLayerSet {
        &self.image_layers
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                x: f32, y: f32, millis: u32) {
        self.image.draw(renderer, scale_x, scale_y, x, y, millis);
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorBuilder {
    pub id: String,
    pub name: String,
    pub race: String,
    pub sex: Option<Sex>,
    pub portrait: Option<String>,
    pub attributes: AttributeList,
    pub player: Option<bool>,
    pub images: HashMap<ImageLayer, String>,
    pub hue: Option<f32>,
    pub hair_color: Option<Color>,
    pub skin_color: Option<Color>,
    pub items: Option<Vec<String>>,
    pub equipped: Option<Vec<u32>>,
    pub levels: HashMap<String, u32>,
}

impl ResourceBuilder for ActorBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ActorBuilder, Error> {
        let resource: ActorBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ActorBuilder, Error> {
        let resource: Result<ActorBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
