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

mod resource_builder_set;
pub use self::resource_builder_set::{
    read_builder, read_builders, read_single_resource, read_single_resource_path, read_to_string,
    write_json_to_file, write_to_file,
};

pub mod sound_set;
pub use self::sound_set::{SoundSetBuilder, SoundSet};

mod spritesheet;
pub use self::spritesheet::Sprite;
pub use self::spritesheet::Spritesheet;

mod font;
pub use self::font::Font;

pub mod yaml_resource_set;
pub use self::yaml_resource_set::YamlResourceKind;
pub use self::yaml_resource_set::YamlResourceSet;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use serde::{de, Deserialize, Deserializer};

use crate::config::Config;
use crate::io::SoundSource;
use crate::image::{
    AnimatedImage, ComposedImage, EmptyImage, Image, SimpleImage, TimerImage, WindowImage,
};
use crate::resource::resource_builder_set::ResourceBuilderSet;
use crate::ui::{Theme, ThemeSet};
use crate::util::{self, invalid_data_error};

thread_local! {
    static RESOURCE_SET: RefCell<ResourceSet> = RefCell::new(ResourceSet::default());
}

#[derive(Default)]
pub struct ResourceSet {
    pub(crate) themes: ThemeSet,
    pub(crate) images: HashMap<String, Rc<dyn Image>>,
    pub(crate) spritesheets: HashMap<String, Rc<Spritesheet>>,
    pub(crate) fonts: HashMap<String, Rc<Font>>,
    pub(crate) sound_sets: HashMap<String, Rc<SoundSet>>,
}

impl ResourceSet {
    pub fn load_resources(mut dirs: Vec<String>) -> Result<YamlResourceSet, Error> {
        if dirs.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Must specify at least \
                 a root data directory to load resources",
            ));
        }

        let yaml_start = std::time::Instant::now();
        let root = dirs.remove(0);
        let path = Path::new(&root);
        let mut yaml = YamlResourceSet::new(path)?;

        for dir in dirs {
            let path = Path::new(&dir);
            yaml.append(path);
        }

        let dir_val = serde_yaml::Value::String(yaml_resource_set::DIRECTORY_VAL_STR.to_string());
        let file_val = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());
        for (key, map) in yaml.resources.iter() {
            for (id, map) in map.iter() {
                trace!(
                    "{:?}: {}, dirs: {:?}, files: {:?}",
                    key,
                    id,
                    map.get(&dir_val),
                    map.get(&file_val)
                );
            }
        }

        log::info!("  Loaded YAML in {}s", util::format_elapsed_secs(yaml_start.elapsed()));

        let builder_start = std::time::Instant::now();
        let builder_set = ResourceBuilderSet::from_yaml(&mut yaml)?;
        log::info!("  Loaded Builders in {}s", util::format_elapsed_secs(builder_start.elapsed()));

        let res_start = std::time::Instant::now();
        ResourceSet::load_builders(builder_set)?;
        log::info!("  Built resources in {}s", util::format_elapsed_secs(res_start.elapsed()));

        Ok(yaml)
    }

    fn load_builders(builder_set: ResourceBuilderSet) -> Result<(), Error> {
        debug!("Creating resource set from parsed data.");

        RESOURCE_SET.with(|resource_set| {
            let mut set = resource_set.borrow_mut();
            set.sound_sets.clear();
            set.images.clear();
            set.spritesheets.clear();
            set.fonts.clear();

            set.themes = builder_set.theme_builder.create_theme_set()?;

            let sound_start = std::time::Instant::now();
            for (id, sounds) in builder_set.sound_set_builders {
                insert_if_ok_boxed("sound_set", id, SoundSet::new(sounds), &mut set.sound_sets);
            }
            info!("    Loaded sounds in {}s", util::format_elapsed_secs(sound_start.elapsed()));

            let sprite_start = std::time::Instant::now();
            for (id, sheet) in builder_set.spritesheet_builders {
                insert_if_ok_boxed(
                    "spritesheet",
                    id,
                    Spritesheet::new(sheet, &mut set),
                    &mut set.spritesheets,
                );
            }
            info!("    Loaded sprites in {}s", util::format_elapsed_secs(sprite_start.elapsed()));

            let font_start = std::time::Instant::now();
            for (id, font) in builder_set.font_builders {
                insert_if_ok_boxed("font", id, Font::new(font), &mut set.fonts);
            }
            info!("    Loaded fonts in {}s", util::format_elapsed_secs(font_start.elapsed()));

            if !set.fonts.contains_key(&Config::default_font()) {
                return invalid_data_error(&format!(
                    "Default font '{}' is not defined.",
                    Config::default_font()
                ));
            }

            let image_start = std::time::Instant::now();

            let empty = Rc::new(EmptyImage {});
            set.images.insert(empty.id(), empty);

            for (id, image) in builder_set.simple_builders {
                insert_if_ok_boxed(
                    "image",
                    id,
                    SimpleImage::generate(image, &set),
                    &mut set.images,
                );
            }

            for (id, image) in builder_set.composed_builders {
                insert_if_ok_boxed(
                    "image",
                    id,
                    ComposedImage::generate(image, &mut set),
                    &mut set.images,
                );
            }

            for (id, image) in builder_set.timer_builders {
                insert_if_ok_boxed(
                    "image",
                    id,
                    TimerImage::generate(image, &set.images),
                    &mut set.images,
                );
            }

            for (id, image) in builder_set.animated_builders {
                insert_if_ok_boxed(
                    "image",
                    id,
                    AnimatedImage::generate(image, &set.images),
                    &mut set.images,
                );
            }

            for (id, image) in builder_set.window_builders {
                insert_if_ok_boxed(
                    "image",
                    id,
                    WindowImage::generate(image, &set),
                    &mut set.images,
                );
            }

            info!("    Loaded images in {}s", util::format_elapsed_secs(image_start.elapsed()));

            Ok(())
        })
    }

    pub fn image_else_empty(id: &str) -> Rc<dyn Image> {
        RESOURCE_SET.with(|r| match get_resource(id, &r.borrow().images) {
            None => {
                warn!("No image with id '{}' found", id);
                get_resource("empty", &r.borrow().images).unwrap()
            }
            Some(ref image) => Rc::clone(image),
        })
    }

    pub fn empty_image() -> Rc<dyn Image> {
        RESOURCE_SET
            .with(|r| get_resource("empty", &r.borrow().images))
            .unwrap()
    }

    pub fn default_font() -> Rc<Font> {
        RESOURCE_SET
            .with(|r| get_resource(&Config::default_font(), &r.borrow().fonts))
            .unwrap()
    }

    pub fn spritesheet(id: &str) -> Option<Rc<Spritesheet>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().spritesheets))
    }

    pub fn panic_or_sprite(id: &str) -> Rc<Sprite> {
        RESOURCE_SET.with(|r| match r.borrow().sprite_internal(id) {
            Ok(sprite) => sprite,
            Err(e) => {
                panic!("Unable to find sprite: '{id}': {e}");
            }
        })
    }

    pub fn sprite(id: &str) -> Result<Rc<Sprite>, Error> {
        RESOURCE_SET.with(|r| r.borrow().sprite_internal(id))
    }

    pub fn default_theme() -> Rc<Theme> {
        RESOURCE_SET.with(|r| Rc::clone(r.borrow().themes.default_theme()))
    }

    pub fn compute_theme_id(parent_id: &str, id: &str) -> String {
        RESOURCE_SET.with(|r| r.borrow().themes.compute_theme_id(parent_id, id))
    }

    pub fn has_theme(id: &str) -> bool {
        RESOURCE_SET.with(|r| r.borrow().themes.contains(id))
    }

    pub fn theme(id: &str) -> Rc<Theme> {
        RESOURCE_SET.with(|r| Rc::clone(r.borrow().themes.get(id)))
    }

    pub fn font(id: &str) -> Option<Rc<Font>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().fonts))
    }

    pub fn image(id: &str) -> Option<Rc<dyn Image>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().images))
    }

    pub fn sound(id: &str) -> Result<SoundSource, Error> {
        RESOURCE_SET.with(|r| r.borrow().sound_internal(id))
    }

    fn sound_internal(&self, id: &str) -> Result<SoundSource, Error> {
        let split_index = match id.find('/') {
            None => return invalid_data_error("Sound must be {SET_ID}/{SOUND_ID}"),
            Some(idx) => idx,
        };

        let (set_id, sound_id) = id.split_at(split_index);
        if sound_id.is_empty() {
            return invalid_data_error("Sound must be {SET_ID}/{SOUND_ID}");
        }

        let sound_id = &sound_id[1..];

        let set = match self.sound_sets.get(set_id) {
            None => {
                return invalid_data_error(&format!("Unable to locate sound set '{set_id}'"));
            }, Some(set) => set,
        };

        let sound = match set.get(sound_id) {
            None => {
                return invalid_data_error(&format!("Unable to locate sound '{sound_id}' in set '{set_id}'"));
            }, Some(sound) => sound.clone(),
        };

        Ok(sound)
    }

    /// Parses the `id` string to get a sprite from a spritesheet.  The string
    /// must be of the form {SPRITE_SHEET_ID}/{SPRITE_ID}
    pub fn sprite_internal(&self, id: &str) -> Result<Rc<Sprite>, Error> {
        let format_error = invalid_data_error(
            "Image display must be \
             of format {SHEET_ID}/{SPRITE_ID}",
        );

        let split_index = match id.find('/') {
            None => return format_error,
            Some(index) => index,
        };

        let (spritesheet_id, sprite_id) = id.split_at(split_index);
        if sprite_id.is_empty() {
            return format_error;
        }
        let sprite_id = &sprite_id[1..];

        let sheet = match self.spritesheets.get(spritesheet_id) {
            None => {
                return invalid_data_error(&format!(
                    "Unable to locate spritesheet '{spritesheet_id}'"
                ));
            }
            Some(sheet) => sheet,
        };

        let sprite = match sheet.sprites.get(sprite_id) {
            None => {
                return invalid_data_error(&format!(
                    "Unable to locate sprite '{sprite_id}' in spritesheet '{spritesheet_id}'"
                ));
            }
            Some(sprite) => Rc::clone(sprite),
        };

        Ok(sprite)
    }
}

pub fn all_resources<V: ?Sized>(map: &HashMap<String, Rc<V>>) -> Vec<Rc<V>> {
    map.iter().map(|ref res| Rc::clone(res.1)).collect()
}

pub fn get_resource<V: ?Sized>(id: &str, map: &HashMap<String, Rc<V>>) -> Option<Rc<V>> {
    map.get(id).map(Rc::clone)
}

pub fn insert_if_ok<K: Eq + Hash + Display, V>(
    type_str: &str,
    key: K,
    val: Result<V, Error>,
    map: &mut HashMap<K, Rc<V>>,
) {
    trace!(
        "Inserting resource of type {} with key {} into resource set.",
        type_str,
        key
    );
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => {
            (*map).insert(key, Rc::new(v));
        }
    };
}

fn insert_if_ok_boxed<K: Eq + Hash + Display, V: ?Sized>(
    type_str: &str,
    key: K,
    val: Result<Rc<V>, Error>,
    map: &mut HashMap<K, Rc<V>>,
) {
    trace!(
        "Inserting resource of type {} with key {} into resource set.",
        type_str,
        key
    );
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => {
            (*map).insert(key, Rc::clone(&v));
        }
    };
}

fn warn_on_insert<K: Display>(type_str: &str, key: K, error: Error) {
    warn!("Error in {} with id '{}'", type_str, key);
    warn!("{}", error);
}

pub fn subdirs<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Error> {
    let mut result = Vec::new();

    let dir_entries = fs::read_dir(path)?;

    for entry in dir_entries {
        let entry = entry?;

        if !entry.path().is_dir() {
            continue;
        }

        result.push(entry.path());
    }

    Ok(result)
}

pub fn deserialize_image<'de, D>(deserializer: D) -> Result<Rc<dyn Image>, D::Error>
where
    D: Deserializer<'de>,
{
    let id = String::deserialize(deserializer)?;
    match ResourceSet::image(&id) {
        None => Err(de::Error::custom(format!(
            "No image with ID '{id}' found"
        ))),
        Some(image) => Ok(image),
    }
}
