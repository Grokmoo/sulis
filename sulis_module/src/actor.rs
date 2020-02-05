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
use std::fmt;
use std::io::Error;
use std::rc::Rc;

use crate::rules::AttributeList;
use sulis_core::image::{Image, LayeredImage};
use sulis_core::io::GraphicsRenderer;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::Color;
use sulis_core::util::unable_to_create_error;

use crate::{
    AITemplate, Ability, Class, Conversation, ImageLayer, ImageLayerSet, InventoryBuilder,
    LootList, Module, Race, RaceBuilder,
};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Faction {
    Friendly,
    Hostile,
    Neutral,
}

impl Faction {
    pub fn iter() -> impl Iterator<Item = &'static Faction> {
        [Faction::Friendly, Faction::Hostile, Faction::Neutral].iter()
    }

    pub fn option_from_str(val: &str) -> Option<Faction> {
        match val {
            "Hostile" => Some(Faction::Hostile),
            "Neutral" => Some(Faction::Neutral),
            "Friendly" => Some(Faction::Friendly),
            _ => None,
        }
    }

    pub fn to_str(self) -> String {
        match &self {
            Faction::Hostile => "Hostile",
            Faction::Neutral => "Neutral",
            Faction::Friendly => "Friendly",
        }
        .to_owned()
    }

    pub fn is_hostile(self, other: Faction) -> bool {
        if self == Faction::Neutral {
            return false;
        }
        if other == Faction::Neutral {
            return false;
        }

        self != other
    }

    pub fn is_friendly(self, other: Faction) -> bool {
        if self == Faction::Neutral {
            return false;
        }
        if other == Faction::Neutral {
            return false;
        }

        self == other
    }
}

#[derive(Debug, Clone)]
pub struct Reward {
    pub xp: u32,
    pub loot: Option<Rc<LootList>>,
    pub loot_chance: u32,
}

#[derive(Debug, Clone)]
pub struct OwnedAbility {
    pub ability: Rc<Ability>,
    pub level: u32,
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

impl Sex {
    pub fn iter() -> impl Iterator<Item = &'static Sex> {
        [Sex::Male, Sex::Female].iter()
    }
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Actor {
    pub id: String,
    pub name: String,
    faction: Faction,
    pub conversation: Option<Rc<Conversation>>,
    pub portrait: Option<Rc<dyn Image>>,
    pub race: Rc<Race>,
    pub sex: Sex,
    pub attributes: AttributeList,
    pub inventory: InventoryBuilder,
    pub xp: u32,
    pub total_level: u32,
    pub levels: Vec<(Rc<Class>, u32)>,
    pub hue: Option<f32>,
    pub hair_color: Option<Color>,
    pub skin_color: Option<Color>,
    image_layers: ImageLayerSet,
    image: LayeredImage,

    // stored directly from the builder, solely for convenience
    pub builder_images: HashMap<ImageLayer, String>,

    pub reward: Option<Reward>,
    pub abilities: Vec<OwnedAbility>,

    pub ai: Option<Rc<AITemplate>>,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn from(
        other: &Actor,
        class_to_add: Option<(Rc<Class>, u32)>,
        xp: u32,
        abilities_to_add: Vec<Rc<Ability>>,
        abilities_to_remove: Vec<String>,
        inventory: InventoryBuilder,
    ) -> Actor {
        let mut levels = other.levels.clone();
        let mut total_level = other.total_level;
        if let Some((class_to_add, amount)) = class_to_add {
            let mut added = false;
            for &mut (ref existing_class, ref mut level) in levels.iter_mut() {
                if *existing_class == class_to_add {
                    *level += amount;
                    added = true;
                }
            }

            if !added {
                levels.push((class_to_add, amount));
            }
            total_level += 1;
        }

        let image_layers = other.image_layers.clone();
        let images_list = image_layers.get_list(other.sex, other.hair_color, other.skin_color);
        let image = LayeredImage::new(images_list, other.hue);

        let mut abilities = other.abilities.clone();
        for ability in abilities_to_add {
            let mut upgraded = false;
            for owned_ability in abilities.iter_mut() {
                if Rc::ptr_eq(&owned_ability.ability, &ability) {
                    owned_ability.level += 1;
                    upgraded = true;
                    break;
                }
            }

            if !upgraded {
                abilities.push(OwnedAbility { ability, level: 0 });
            }
        }

        for ability in abilities_to_remove {
            abilities.retain(|owned| owned.ability.id != ability);
        }

        Actor {
            id: other.id.to_string(),
            name: other.name.to_string(),
            faction: other.faction,
            conversation: other.conversation.clone(),
            portrait: other.portrait.clone(),
            race: Rc::clone(&other.race),
            sex: other.sex,
            attributes: other.attributes,
            inventory,
            xp,
            total_level,
            levels,
            hue: other.hue,
            hair_color: other.hair_color,
            skin_color: other.skin_color,
            image_layers,
            image,
            builder_images: other.builder_images.clone(),
            reward: other.reward.clone(),
            abilities,
            ai: other.ai.clone(),
        }
    }

    pub fn new(builder: ActorBuilder, resources: &mut Module) -> Result<Actor, Error> {
        let race = if let Some(race_id) = builder.race {
            match resources.races.get(&race_id) {
                None => {
                    warn!("No match found for race '{}'", race_id);
                    return unable_to_create_error("actor", &builder.id);
                }
                Some(race) => Rc::clone(race),
            }
        } else if let Some(race_builder) = builder.inline_race {
            let race = Rc::new(Race::new(race_builder, resources)?);
            trace!("Inserting inline race with ID {} into module.", race.id);
            resources.races.insert(race.id.clone(), Rc::clone(&race));
            race
        } else {
            warn!("Must specify either race or inline race.");
            return unable_to_create_error("actor", &builder.id);
        };

        let conversation = match builder.conversation {
            None => None,
            Some(ref convo_id) => Some(match resources.conversations.get(convo_id) {
                None => {
                    warn!("No match found for conversation '{}'", convo_id);
                    return unable_to_create_error("actor", &builder.id);
                }
                Some(convo) => Rc::clone(convo),
            }),
        };

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
                }
                Some(class) => Rc::clone(class),
            };

            total_level += level;
            levels.push((class, level));
        }

        let portrait = match builder.portrait {
            None => None,
            Some(ref image) => match ResourceSet::image(image) {
                None => {
                    warn!("Unable to find image for portrait '{}'", image);
                    return unable_to_create_error("actor", &builder.id);
                }
                Some(image) => Some(image),
            },
        };

        let image_layers =
            ImageLayerSet::merge(race.default_images(), sex, builder.images.clone())?;
        let images_list = image_layers.get_list(sex, builder.hair_color, builder.skin_color);
        let image = LayeredImage::new(images_list, builder.hue);

        let reward = match builder.reward {
            None => None,
            Some(reward) => {
                let xp = reward.xp;
                let loot = match reward.loot {
                    None => None,
                    Some(id) => Some(match resources.loot_lists.get(&id) {
                        None => {
                            warn!("No loot list found with id '{}'", id);
                            return unable_to_create_error("actor", &builder.id);
                        }
                        Some(list) => Rc::clone(list),
                    }),
                };

                Some(Reward {
                    xp,
                    loot,
                    loot_chance: reward.loot_chance.unwrap_or(100),
                })
            }
        };

        let mut abilities: Vec<OwnedAbility> = Vec::new();
        for ability_id in builder.abilities {
            let ability = match resources.abilities.get(&ability_id) {
                None => {
                    warn!("No ability found for '{}'", ability_id);
                    return unable_to_create_error("actor", &builder.id);
                }
                Some(ref ability) => Rc::clone(ability),
            };

            let mut upgrade = false;
            for owned_ability in abilities.iter_mut() {
                if Rc::ptr_eq(&owned_ability.ability, &ability) {
                    owned_ability.level += 1;
                    upgrade = true;
                    break;
                }
            }

            if !upgrade {
                abilities.push(OwnedAbility { ability, level: 0 });
            }
        }

        let ai = match builder.ai {
            None => None,
            Some(id) => match resources.ai_templates.get(&id) {
                None => {
                    warn!("No AI template found with id '{}'", id);
                    return unable_to_create_error("actor", &builder.id);
                }
                Some(ref ai) => Some(Rc::clone(ai)),
            },
        };

        Ok(Actor {
            id: builder.id,
            name: builder.name,
            conversation,
            faction: builder.faction.unwrap_or(Faction::Hostile),
            portrait,
            race,
            sex,
            attributes: builder.attributes,
            levels,
            total_level,
            xp: builder.xp.unwrap_or(0),
            inventory: builder.inventory,
            reward,
            image_layers,
            image,
            builder_images: builder.images,
            hue: builder.hue,
            skin_color: builder.skin_color,
            hair_color: builder.hair_color,
            abilities,
            ai,
        })
    }

    pub fn faction(&self) -> Faction {
        self.faction
    }

    pub fn levels(&self, other_class: &Rc<Class>) -> u32 {
        for &(ref class, level) in self.levels.iter() {
            if class == other_class {
                return level;
            }
        }

        0
    }

    pub fn ability_level(&self, id: &str) -> Option<u32> {
        for ability in self.abilities.iter() {
            if ability.ability.id == id {
                return Some(ability.level);
            }
        }

        None
    }

    pub fn has_ability_with_id(&self, id: &str) -> bool {
        for ability in self.abilities.iter() {
            if ability.ability.id == id {
                return true;
            }
        }

        false
    }

    pub fn has_ability(&self, other: &Rc<Ability>) -> bool {
        for ability in self.abilities.iter() {
            if Rc::ptr_eq(&ability.ability, other) {
                return true;
            }
        }

        false
    }

    pub fn base_class(&self) -> Rc<Class> {
        Rc::clone(&self.levels[0].0)
    }

    pub fn image_layers(&self) -> &ImageLayerSet {
        &self.image_layers
    }

    pub fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        scale_x: f32,
        scale_y: f32,
        x: f32,
        y: f32,
        millis: u32,
    ) {
        self.image.draw(renderer, scale_x, scale_y, x, y, millis);
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RewardBuilder {
    pub xp: u32,
    pub loot: Option<String>,
    pub loot_chance: Option<u32>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorBuilder {
    pub id: String,
    pub name: String,

    pub race: Option<String>,

    #[serde(skip_serializing)]
    pub inline_race: Option<RaceBuilder>,

    pub sex: Option<Sex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portrait: Option<String>,
    pub attributes: AttributeList,
    pub conversation: Option<String>,
    pub faction: Option<Faction>,

    #[serde(default)]
    pub images: HashMap<ImageLayer, String>,

    pub hue: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hair_color: Option<Color>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_color: Option<Color>,
    #[serde(default)]
    pub inventory: InventoryBuilder,
    pub levels: HashMap<String, u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub xp: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<RewardBuilder>,
    pub abilities: Vec<String>,
    pub ai: Option<String>,
}
