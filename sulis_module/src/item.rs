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
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::serde_yaml;
use sulis_core::util::unable_to_create_error;
use sulis_rules::{ArmorKind, WeaponKind};

use {Equippable, ImageLayer, Module};

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Slot {
    Cloak,
    Head,
    Torso,
    Hands,
    HeldMain,
    HeldOff,
    Legs,
    Feet,
    Waist,
    Neck,
    FingerMain,
    FingerOff,
}

impl Slot {
    pub fn iter() -> ::std::slice::Iter<'static, Slot> {
        SLOTS_LIST.iter()
    }
}

use self::Slot::*;

// The sort order of this list is important
const SLOTS_LIST: [Slot; 12] = [Cloak, Feet, Legs, Torso, Hands, Head, HeldMain, HeldOff, Waist,
                                Neck, FingerMain, FingerOff];

#[derive(Debug, Deserialize)]
pub enum ItemKind {
    Armor { kind: ArmorKind },
    Weapon { kind: WeaponKind },
    Other,
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub icon: Rc<Image>,
    pub equippable: Option<Equippable>,
    image: Option<HashMap<ImageLayer, Rc<Image>>>,
    alternate_image: Option<HashMap<ImageLayer, Rc<Image>>>,
    pub value: i32,
    pub weight: i32,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.id == other.id
    }
}

fn build_hash_map(id: &str, input: Option<HashMap<ImageLayer, String>>)
    -> Result<Option<HashMap<ImageLayer, Rc<Image>>>, Error> {
    match input {
        Some(input_images) => {
            let mut output = HashMap::new();
            for (layer, image_str) in input_images {
                let image = match ResourceSet::get_image(&image_str) {
                    None => {
                        warn!("No image found for image '{}'", image_str);
                        return unable_to_create_error("item", id);
                    }, Some(image) => image,
                };

                output.insert(layer, image);
            }

            Ok(Some(output))
        }, None => Ok(None),
    }
}

impl Item {
    pub fn new(builder: ItemBuilder, _module: &Module) -> Result<Item, Error> {
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return unable_to_create_error("item", &builder.id);
            },
            Some(icon) => icon
        };


        if let &Some(ref equippable) = &builder.equippable {
            if let Some(ref attack) = equippable.attack {
                if attack.damage.kind.is_none() {
                    warn!("Kind must be specified for attack damage.");
                    return unable_to_create_error("item", &builder.id);
                }
            }
        }

        let images = build_hash_map(&builder.id, builder.image)?;
        let alt_images = build_hash_map(&builder.id, builder.alternate_image)?;

        Ok(Item {
            id: builder.id,
            icon: icon,
            image: images,
            kind: builder.kind.unwrap_or(ItemKind::Other),
            alternate_image: alt_images,
            name: builder.name,
            equippable: builder.equippable,
            value: builder.value as i32,
            weight: builder.weight as i32,
        })
    }

    pub fn alt_image_iter(&self) -> Option<Iter<ImageLayer, Rc<Image>>> {
        match self.alternate_image {
            None => None,
            Some(ref image) => Some(image.iter()),
        }
    }

    pub fn image_iter(&self) -> Option<Iter<ImageLayer, Rc<Image>>> {
        match self.image {
            None => None,
            Some(ref image) => Some(image.iter()),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemBuilder {
    pub id: String,
    pub name: String,
    kind: Option<ItemKind>,
    pub icon: String,
    pub equippable: Option<Equippable>,
    pub image: Option<HashMap<ImageLayer, String>>,
    pub alternate_image: Option<HashMap<ImageLayer, String>>,
    value: u32,
    weight: u32,
}

impl ResourceBuilder for ItemBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_yaml(data: &str) -> Result<ItemBuilder, Error> {
        let resource: Result<ItemBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}

pub fn format_item_value(value: i32) -> String {
    let display_factor = Module::rules().item_value_display_factor;

    let value_f = value as f32 / display_factor;
    format!("{:.*}", 1, value_f)
}

pub fn format_item_weight(weight: i32) -> String {
    let display_factor = Module::rules().item_weight_display_factor;

    let weight_f = weight as f32 / display_factor;
    format!("{:.*}", 2, weight_f)
}
