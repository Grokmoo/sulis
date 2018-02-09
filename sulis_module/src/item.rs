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

use std::slice::Iter;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_core::util::invalid_data_error;

use {Equippable, ImageLayer};

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Slot {
    Head,
    Torso,
    Hands,
    HeldMain,
    HeldOff,
    Legs,
    Feet,
}

impl Slot {
    pub fn iter() -> Iter<'static, Slot> {
        SLOTS_LIST.iter()
    }
}

use self::Slot::*;

const SLOTS_LIST: [Slot; 7] = [Feet, Legs, Torso, Hands, Head, HeldOff, HeldMain];

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub icon: Rc<Image>,
    pub equippable: Option<Equippable>,
    pub image: HashMap<ImageLayer, Rc<Image>>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.id == other.id
    }
}

impl Item {
    pub fn new(builder: ItemBuilder) -> Result<Item, Error> {
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Unable to create item '{}'", builder.id)));
            },
            Some(icon) => icon
        };

        let mut images = HashMap::new();
        for (layer, image_str) in builder.image {
            let image = match ResourceSet::get_image(&image_str) {
                None => {
                    warn!("No image found for image '{}'", image_str);
                    return invalid_data_error(&format!("Unable to create item '{}'", builder.id));
                }, Some(image) => image,
            };

            images.insert(layer, image);
        }

        Ok(Item {
            id: builder.id,
            icon: icon,
            image: images,
            name: builder.name,
            equippable: builder.equippable,
        })
    }

    pub fn image_for_layer(&self, layer: ImageLayer) -> Option<&Rc<Image>> {
        self.image.get(&layer)
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemBuilder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub equippable: Option<Equippable>,
    pub image: HashMap<ImageLayer, String>,
}

impl ResourceBuilder for ItemBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ItemBuilder, Error> {
        let resource: ItemBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ItemBuilder, Error> {
        let resource: Result<ItemBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
