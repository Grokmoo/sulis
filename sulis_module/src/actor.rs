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

use sulis_core::image::{Image, LayeredImage};
use sulis_core::io::DrawList;
use sulis_core::resource::ResourceBuilder;
use sulis_core::ui::animation_state;
use sulis_core::util::invalid_data_error;
use sulis_core::{serde_json, serde_yaml};
use sulis_rules::AttributeList;

use {Class, ImageLayer, ImageLayerSet, Item, Module, Race};

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    pub race: Rc<Race>,
    pub sex: Sex,
    pub attributes: AttributeList,
    pub items: Vec<Rc<Item>>,
    pub to_equip: Vec<usize>,
    pub levels: Vec<(Rc<Class>, u32)>,

    hue: Option<f32>,
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

        let image_layers = ImageLayerSet::merge(race.default_images(), sex, builder.images)?;
        let images_list = image_layers.get_list(sex);
        let image = LayeredImage::new(images_list);

        Ok(Actor {
            id: builder.id,
            name: builder.name,
            player: builder.player.unwrap_or(false),
            race,
            sex,
            attributes: builder.attributes.unwrap_or(AttributeList::default()),
            levels,
            items,
            to_equip,
            image_layers,
            image,
            hue: builder.hue,
        })
    }

    pub fn check_add_swap_hue(&self, draw_list: &mut DrawList) {
        match self.hue {
            Some(hue) => draw_list.set_swap_hue(hue),
            None => (),
        }
    }

    pub fn image_layers(&self) -> &ImageLayerSet {
        &self.image_layers
    }

    pub fn append_to_draw_list_i32(&self, draw_list: &mut DrawList, x: i32, y: i32, millis: u32) {
        self.append_to_draw_list(draw_list, x as f32, y as f32, millis);
    }

    fn append_to_draw_list(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        let size = self.race.size.size as f32;
        self.image.append_to_draw_list(draw_list, &animation_state::NORMAL,
                                       x, y, size, size, millis);
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActorBuilder {
    id: String,
    name: String,
    race: String,
    sex: Option<Sex>,
    attributes: Option<AttributeList>,
    player: Option<bool>,
    images: HashMap<ImageLayer, String>,
    hue: Option<f32>,
    items: Option<Vec<String>>,
    equipped: Option<Vec<u32>>,
    levels: HashMap<String, u32>,
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
