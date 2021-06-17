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

use crate::rules::{bonus::AttackBuilder, BonusList, Slot};
use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::Color;
use sulis_core::util::{gen_rand, unable_to_create_error, Point};

use crate::actor::Sex;

use crate::{ImageLayer, ImageLayerSet, Module, ObjectSize, Prop};

#[derive(Debug)]
pub struct Race {
    pub id: String,
    pub name: String,
    pub description: String,
    pub movement_rate: f32,
    pub move_anim_rate: f32,
    pub pc_death_prop: Option<Rc<Prop>>,
    pub size: Rc<ObjectSize>,
    pub base_stats: BonusList,
    pub base_attack: AttackBuilder,
    pub hair_selections: Vec<String>,
    pub beard_selections: Vec<String>,
    pub portrait_selections: Vec<String>,
    pub male_random_names: Vec<String>,
    pub female_random_names: Vec<String>,
    pub disabled_slots: Vec<Slot>,
    pub hair_colors: Vec<Color>,
    pub skin_colors: Vec<Color>,
    pub ticker_offset: (f32, f32),
    default_images: ImageLayerSet,
    image_layer_offsets: HashMap<ImageLayer, (f32, f32)>,
    image_layer_postfix: HashMap<Sex, String>,

    editor_creator_images: Vec<(ImageLayer, Vec<Rc<dyn Image>>)>,
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
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
            }
            Some(size) => Rc::clone(size),
        };

        let mut offsets = HashMap::new();
        let scale = builder.image_layer_offset_scale as f32;
        for (layer, p) in builder.image_layer_offsets {
            offsets.insert(layer, (p.x as f32 / scale, p.y as f32 / scale));
        }

        let default_images = if !builder.default_images.is_empty() {
            let default_images = builder.default_images.clone();
            let images = Sex::iter().map(|s| (*s, default_images.clone())).collect();
            ImageLayerSet::new(images)
        } else if !builder.default_images_by_sex.is_empty() {
            ImageLayerSet::new(builder.default_images_by_sex)
        } else {
            warn!("Must specify either default_images or default_images_by_sex");
            return unable_to_create_error("race", &builder.id);
        }?;

        if builder.base_attack.damage.kind.is_none() {
            warn!("Attack must always have a damage kind specified.");
            return unable_to_create_error("race", &builder.id);
        }

        let mut hair_colors = Vec::new();
        let mut skin_colors = Vec::new();
        for color_str in builder.hair_colors.unwrap_or_default().iter() {
            hair_colors.push(Color::from_string(color_str));
        }

        for color_str in builder.skin_colors.unwrap_or_default().iter() {
            skin_colors.push(Color::from_string(color_str));
        }

        let mut editor_creator_images = Vec::new();
        for (layer, vec) in builder.editor_creator_images {
            let mut images = Vec::new();
            for image_id in vec {
                match ResourceSet::image(&image_id) {
                    None => {
                        warn!("No image found with id '{}'", image_id);
                        return unable_to_create_error("race", &builder.id);
                    }
                    Some(image) => images.push(image),
                }
            }
            editor_creator_images.push((layer, images));
        }

        let pc_death_prop = match builder.pc_death_prop {
            None => None,
            Some(id) => match module.props.get(&id) {
                None => {
                    warn!("No prop found with id '{}' for pc_death_prop", id);
                    return unable_to_create_error("race", &builder.id);
                }
                Some(prop) => Some(Rc::clone(prop)),
            },
        };

        Ok(Race {
            id: builder.id,
            name: builder.name,
            description: builder.description,
            movement_rate: builder.movement_rate,
            move_anim_rate: builder.move_anim_rate,
            size,
            disabled_slots: builder.disabled_slots,
            base_stats: builder.base_stats,
            base_attack: builder.base_attack,
            default_images,
            image_layer_offsets: offsets,
            image_layer_postfix: builder.image_layer_postfix,
            hair_selections: builder.hair_selections.unwrap_or_default(),
            beard_selections: builder.beard_selections.unwrap_or_default(),
            portrait_selections: builder.portrait_selections.unwrap_or_default(),
            male_random_names: builder.male_random_names,
            female_random_names: builder.female_random_names,
            hair_colors,
            skin_colors,
            ticker_offset: builder.ticker_offset,
            editor_creator_images,
            pc_death_prop,
        })
    }

    pub fn random_name(&self, sex: &Sex) -> &str {
        let names = match sex {
            Sex::Male => &self.male_random_names,
            Sex::Female => &self.female_random_names,
        };

        if names.is_empty() {
            return "";
        }

        if names.len() == 1 {
            return &names[0];
        }

        let index = gen_rand(0, names.len());
        &names[index]
    }

    pub fn has_editor_creator_images(&self) -> bool {
        !self.editor_creator_images.is_empty()
    }

    pub fn editor_creator_images(&self) -> Vec<(ImageLayer, Vec<Rc<dyn Image>>)> {
        self.editor_creator_images.clone()
    }

    pub fn image_for_sex(&self, sex: Sex, image: &Rc<dyn Image>) -> Rc<dyn Image> {
        match self.image_layer_postfix.get(&sex) {
            None => Rc::clone(image),
            Some(postfix) => {
                let id = image.id() + postfix;
                match ResourceSet::image(&id) {
                    None => Rc::clone(image),
                    Some(ref new_image) => Rc::clone(new_image),
                }
            }
        }
    }

    pub fn get_image_layer_offset(&self, layer: ImageLayer) -> Option<&(f32, f32)> {
        self.image_layer_offsets.get(&layer)
    }

    pub fn default_images(&self) -> &ImageLayerSet {
        &self.default_images
    }

    pub fn is_disabled(&self, slot: Slot) -> bool {
        for disabled in self.disabled_slots.iter() {
            if *disabled == slot {
                return true;
            }
        }
        false
    }
}

fn float_1() -> f32 { 1.0 }

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RaceBuilder {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub description: String,
    pub size: String,
    pub movement_rate: f32,
    pub base_attack: AttackBuilder,
    pub base_stats: BonusList,
    pub pc_death_prop: Option<String>,
    pub hair_selections: Option<Vec<String>>,
    pub beard_selections: Option<Vec<String>>,
    pub portrait_selections: Option<Vec<String>>,

    #[serde(default="float_1")]
    pub move_anim_rate: f32,

    #[serde(default)]
    pub default_images: HashMap<ImageLayer, String>,

    #[serde(default)]
    pub default_images_by_sex: HashMap<Sex, HashMap<ImageLayer, String>>,

    #[serde(default)]
    pub male_random_names: Vec<String>,

    #[serde(default)]
    pub female_random_names: Vec<String>,

    pub hair_colors: Option<Vec<String>>,
    pub skin_colors: Option<Vec<String>>,
    pub ticker_offset: (f32, f32),
    image_layer_offsets: HashMap<ImageLayer, Point>,
    image_layer_offset_scale: i32,

    #[serde(default)]
    image_layer_postfix: HashMap<Sex, String>,

    #[serde(default)]
    editor_creator_images: HashMap<ImageLayer, Vec<String>>,

    #[serde(default)]
    disabled_slots: Vec<Slot>,
}
