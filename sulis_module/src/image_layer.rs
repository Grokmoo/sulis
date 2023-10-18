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
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;

use crate::actor::Sex;
use crate::Race;
use sulis_core::image::{layered_image::Layer, Image};
use sulis_core::resource::ResourceSet;
use sulis_core::ui::Color;
use sulis_core::util::invalid_data_error;

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum ImageLayer {
    HeldMain,
    HeldOff,
    Ears,
    Hair,
    Beard,
    Head,
    Hands,
    Foreground,
    Torso,
    Legs,
    Feet,
    Background,
    Cloak,
    Shadow,
}

impl FromStr for ImageLayer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ImageLayer::*;
        let val = match s {
            "HeldMain" => HeldMain,
            "HeldOff" => HeldOff,
            "Ears" => Ears,
            "Hair" => Hair,
            "Beard" => Beard,
            "Head" => Head,
            "Hands" => Hands,
            "Foreground" => Foreground,
            "Torso" => Torso,
            "Legs" => Legs,
            "Feet" => Feet,
            "Background" => Background,
            "Cloak" => Cloak,
            "Shadown" => Shadow,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Unable to parse ImageLayer from '{s}'"),
                ));
            }
        };

        Ok(val)
    }
}

use self::ImageLayer::*;
const IMAGE_LAYERS_LIST: [ImageLayer; 14] = [
    Shadow, Cloak, Background, Feet, Legs, Torso, Foreground, Hands, Beard, Hair, Head, Ears,
    HeldOff, HeldMain,
];

impl ImageLayer {
    pub fn iter() -> Iter<'static, ImageLayer> {
        IMAGE_LAYERS_LIST.iter()
    }
}

#[derive(Clone, Debug)]
pub struct ImageLayerSet {
    images: HashMap<Sex, HashMap<ImageLayer, Rc<dyn Image>>>,
}

impl ImageLayerSet {
    pub fn new(
        image_refs: HashMap<Sex, HashMap<ImageLayer, String>>,
    ) -> Result<ImageLayerSet, Error> {
        let mut images = HashMap::new();

        for (sex, layers_map) in image_refs {
            ImageLayerSet::append(&mut images, sex, layers_map)?;
        }

        Ok(ImageLayerSet { images })
    }

    pub fn merge(
        base: &ImageLayerSet,
        sex: Sex,
        append: HashMap<ImageLayer, String>,
    ) -> Result<ImageLayerSet, Error> {
        let mut images = base.images.clone();
        ImageLayerSet::append(&mut images, sex, append)?;

        Ok(ImageLayerSet { images })
    }

    /// Returns a reference to the image for this sex and ImageLayer,
    /// or None if no such image exists.
    pub fn get(&self, sex: Sex, layer: ImageLayer) -> Option<&Rc<dyn Image>> {
        match self.images.get(&sex) {
            None => None,
            Some(sex_map) => sex_map.get(&layer),
        }
    }

    /// Gets the list of images from this ImageLayerSet for the given Sex.
    /// The images are ordered based on the iteration order of ImageLayer
    pub fn get_list(&self, sex: Sex, hair: Option<Color>, skin: Option<Color>) -> Vec<Layer> {
        let mut list = Vec::new();

        match self.images.get(&sex) {
            None => return list,
            Some(sex_map) => {
                for layer in ImageLayer::iter() {
                    let image = match sex_map.get(layer) {
                        None => continue,
                        Some(image) => Rc::clone(image),
                    };

                    let out = Layer::new(0.0, 0.0, get_color(*layer, hair, skin), image);
                    list.push(out);
                }
            }
        }

        list
    }

    /// Gets the list of images from this ImageLayerSet for the given Sex,
    /// with the additional images inserted
    pub fn get_list_with(
        &self,
        sex: Sex,
        race: &Rc<Race>,
        hair: Option<Color>,
        skin: Option<Color>,
        insert: HashMap<ImageLayer, Rc<dyn Image>>,
    ) -> Vec<Layer> {
        let mut list = Vec::new();

        match self.images.get(&sex) {
            Some(sex_map) => {
                for layer in ImageLayer::iter() {
                    let mut base_size = None;
                    if let Some(image) = sex_map.get(layer) {
                        base_size = Some((image.get_width_f32(), image.get_height_f32()));
                    }

                    if insert_for_race_sex(&mut list, &insert, sex, race, *layer, base_size) {
                        continue;
                    }

                    let (x, y) = match race.get_image_layer_offset(*layer) {
                        None => continue,
                        Some((x, y)) => (*x, *y),
                    };
                    if let Some(image) = sex_map.get(layer) {
                        let out = Layer::new(x, y, get_color(*layer, hair, skin), Rc::clone(image));
                        list.push(out);
                    }
                }
            }
            None => {
                for layer in ImageLayer::iter() {
                    insert_for_race_sex(&mut list, &insert, sex, race, *layer, None);
                }
            }
        }

        list
    }

    fn append(
        images: &mut HashMap<Sex, HashMap<ImageLayer, Rc<dyn Image>>>,
        sex: Sex,
        refs: HashMap<ImageLayer, String>,
    ) -> Result<(), Error> {
        let sex_map = images.entry(sex).or_default();

        for (image_layer, image_str) in refs {
            let image = match ResourceSet::image(&image_str) {
                None => {
                    warn!(
                        "Image '{}' not found for layer '{:?}'",
                        image_str, image_layer
                    );
                    return invalid_data_error("Unable to create image_layer_set");
                }
                Some(image) => image,
            };
            sex_map.insert(image_layer, image);
        }

        Ok(())
    }
}

fn get_color(layer: ImageLayer, hair: Option<Color>, skin: Option<Color>) -> Option<Color> {
    match layer {
        Hair | Beard => hair,
        Background | Foreground | Ears => skin,
        _ => None,
    }
}

fn insert_for_race_sex(
    list: &mut Vec<Layer>,
    insert: &HashMap<ImageLayer, Rc<dyn Image>>,
    sex: Sex,
    race: &Rc<Race>,
    layer: ImageLayer,
    base_size: Option<(f32, f32)>,
) -> bool {
    let (mut x, mut y) = match race.get_image_layer_offset(layer) {
        None => return true,
        Some((x, y)) => (*x, *y),
    };

    match insert.get(&layer) {
        Some(image) => {
            if let Some((width_base, height_base)) = base_size {
                x -= (image.get_width_f32() - width_base) / 2.0;
                y -= (image.get_height_f32() - height_base) / 2.0;
            }

            let out = Layer::new(x, y, None, race.image_for_sex(sex, image));
            list.push(out);
            true
        }
        None => false,
    }
}
