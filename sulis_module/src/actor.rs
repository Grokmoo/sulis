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
use sulis_core::util::{unable_to_create_error};
use sulis_core::{serde_json, serde_yaml};
use sulis_rules::AttributeList;

use {Ability, Class, ImageLayer, ImageLayerSet, Item, LootList, Module, Race};

#[derive(Debug, Clone)]
pub struct Reward {
    pub xp: u32,
    pub loot: Option<Rc<LootList>>,
    pub loot_chance: u32,
}

impl Default for Reward {
    fn default() -> Reward {
        Reward {
            xp: 0,
            loot: None,
            loot_chance: 100,
        }
    }
}

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
    pub xp: u32,
    pub total_level: u32,
    pub levels: Vec<(Rc<Class>, u32)>,
    pub hue: Option<f32>,
    pub hair_color: Option<Color>,
    pub skin_color: Option<Color>,
    image_layers: ImageLayerSet,
    image: LayeredImage,

    pub reward: Option<Reward>,
    pub abilities: Vec<Rc<Ability>>,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn from(other: &Actor, class_to_add: Rc<Class>, xp: u32,
                mut abilities_to_add: Vec<Rc<Ability>>) -> Actor {
        let mut added = false;
        let mut levels = other.levels.clone();
        for &mut (ref existing_class, ref mut level) in levels.iter_mut() {
            if *existing_class == class_to_add {
                *level += 1;
                added = true;
            }
        }

        if !added {
            levels.push((class_to_add, 1));
        }

        let image_layers = other.image_layers.clone();
        let images_list = image_layers.get_list(other.sex, other.hair_color, other.skin_color);
        let image = LayeredImage::new(images_list, other.hue);

        let mut abilities = other.abilities.clone();
        abilities.append(&mut abilities_to_add);

        Actor {
            id: other.id.to_string(),
            name: other.name.to_string(),
            player: other.player,
            portrait: other.portrait.clone(),
            race: Rc::clone(&other.race),
            sex: other.sex,
            attributes: other.attributes,
            items: other.items.clone(),
            to_equip: other.to_equip.clone(),
            xp,
            total_level: other.total_level + 1,
            levels,
            hue: other.hue,
            hair_color: other.hair_color,
            skin_color: other.skin_color,
            image_layers,
            image,
            reward: other.reward.clone(),
            abilities,
        }
    }

    pub fn new(builder: ActorBuilder, resources: &Module) -> Result<Actor, Error> {
        let race = match resources.races.get(&builder.race) {
            None => {
                warn!("No match found for race '{}'", builder.race);
                return unable_to_create_error("actor", &builder.id);
            }, Some(race) => Rc::clone(race)
        };

        let mut items: Vec<Rc<Item>> = Vec::new();
        if let Some(builder_items) = builder.items {
            for item_id in builder_items {
                let item = match resources.items.get(&item_id) {
                    None => {
                        warn!("No match found for item ID '{}'", item_id);
                        return unable_to_create_error("actor", &builder.id);
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

        let mut total_level = 0;
        let mut levels: Vec<(Rc<Class>, u32)> = Vec::new();
        for (class_id, level) in builder.levels {
            let class = match resources.classes.get(&class_id) {
                None => {
                    warn!("No match for class '{}'", class_id);
                    return unable_to_create_error("actor", &builder.id);
                }, Some(class) => Rc::clone(class)
            };

            total_level += level;
            levels.push((class, level));
        }

        let mut to_equip: Vec<usize> = Vec::new();
        if let Some(builder_equipped) = builder.equipped {
            for index in builder_equipped {
                let index = index as usize;
                if index >= items.len() {
                    warn!("No item exists to equip at index {}", index);
                    return unable_to_create_error("actor", &builder.id);
                }

                if items[index].equippable.is_none() {
                    warn!("Item at index {}: {} is not equippable.", index, items[index].name);
                    return unable_to_create_error("actor", &builder.id);
                }

                to_equip.push(index);
            }
        }

        let portrait = match builder.portrait {
            None => None,
            Some(ref image) => match ResourceSet::get_image(image) {
                None => {
                    warn!("Unable to find image for portrait '{}'", image);
                    return unable_to_create_error("actor", &builder.id);
                }, Some(image) => Some(image),
            }
        };

        let image_layers = ImageLayerSet::merge(race.default_images(), sex, builder.images)?;
        let images_list = image_layers.get_list(sex, builder.hair_color, builder.skin_color);
        let image = LayeredImage::new(images_list, builder.hue);

        let reward = match builder.reward {
            None => None,
            Some(reward) => {
                let xp = reward.xp;
                let loot = match reward.loot {
                    None => None,
                    Some(id) => {
                        Some(match resources.loot_lists.get(&id) {
                            None => {
                                warn!("No loot list found with id '{}'", id);
                                return unable_to_create_error("actor", &builder.id);
                            }, Some(list) => Rc::clone(list),
                        })
                    }
                };

                Some(Reward {
                    xp,
                    loot,
                    loot_chance: reward.loot_chance.unwrap_or(100),
                })
            }
        };

        let mut abilities = Vec::new();
        if let Some(ref builder_abilities) = builder.abilities {
            for ability_id in builder_abilities {
                let ability = match resources.abilities.get(ability_id) {
                    None => {
                        warn!("No ability found for '{}'", ability_id);
                        return unable_to_create_error("actor", &builder.id);
                    }, Some(ref ability) => Rc::clone(ability),
                };
                abilities.push(ability);
            }
        }

        Ok(Actor {
            id: builder.id,
            name: builder.name,
            player: builder.player.unwrap_or(false),
            portrait,
            race,
            sex,
            attributes: builder.attributes,
            levels,
            total_level,
            xp: builder.xp.unwrap_or(0),
            reward,
            items,
            to_equip,
            image_layers,
            image,
            hue: builder.hue,
            skin_color: builder.skin_color,
            hair_color: builder.hair_color,
            abilities,
        })
    }

    pub fn base_class(&self) -> Rc<Class> {
        Rc::clone(&self.levels[0].0)
    }

    pub fn image_layers(&self) -> &ImageLayerSet {
        &self.image_layers
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                x: f32, y: f32, millis: u32) {
        self.image.draw(renderer, scale_x, scale_y, x, y, millis);
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RewardBuilder {
    xp: u32,
    loot: Option<String>,
    loot_chance: Option<u32>,
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
    pub xp: Option<u32>,
    pub reward: Option<RewardBuilder>,
    pub abilities: Option<Vec<String>>,
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
