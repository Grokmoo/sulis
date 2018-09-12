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
pub use self::resource_builder_set::{read_single_resource, read_single_resource_path,
    read_to_string, write_to_file, write_json_to_file, read_builder, read_builders};

mod spritesheet;
pub use self::spritesheet::Spritesheet;
pub use self::spritesheet::Sprite;

mod font;
pub use self::font::Font;

pub mod yaml_resource_set;
pub use self::yaml_resource_set::YamlResourceSet;
pub use self::yaml_resource_set::YamlResourceKind;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::io::{Error, ErrorKind};
use std::fmt::Display;
use std::hash::Hash;
use std::fs;
use std::path::PathBuf;

use serde_yaml;

use config::Config;
use resource::resource_builder_set::ResourceBuilderSet;
use image::{Image, EmptyImage, SimpleImage, AnimatedImage, ComposedImage, TimerImage};
use util::invalid_data_error;
use ui::Theme;

thread_local! {
    static RESOURCE_SET: RefCell<ResourceSet> = RefCell::new(ResourceSet::new());
}

pub struct ResourceSet {
    pub (crate) theme: Option<Rc<Theme>>,
    pub (crate) images: HashMap<String, Rc<Image>>,
    pub (crate) spritesheets: HashMap<String, Rc<Spritesheet>>,
    pub (crate) fonts: HashMap<String, Rc<Font>>,
}

impl ResourceSet {
    fn new() -> ResourceSet {
        ResourceSet {
            theme: None,
            images: HashMap::new(),
            spritesheets: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn load_resources(mut dirs: Vec<String>) -> Result<YamlResourceSet, Error> {
        if dirs.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Must specify at least \
                                  a root data directory to load resources"));
        }

        let root = dirs.remove(0);
        let theme_dir = format!("{}/theme/", root);
        let path = Path::new(&root);
        let mut yaml = YamlResourceSet::new(&path)?;

        for dir in dirs {
            let path = Path::new(&dir);
            yaml.append(path);
        }

        let dir_val = serde_yaml::Value::String(yaml_resource_set::DIRECTORY_VAL_STR.to_string());
        let file_val = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());
        for (key, map) in yaml.resources.iter() {
            for (id, map) in map.iter() {
                trace!("{:?}: {}, dirs: {:?}, files: {:?}", key, id, map.get(&dir_val),
                    map.get(&file_val));
            }
        }

        let builder_set = ResourceBuilderSet::from_yaml(&mut yaml, &theme_dir)?;
        ResourceSet::load_builders(builder_set)?;

        Ok(yaml)
    }

    fn load_builders(builder_set: ResourceBuilderSet) -> Result<(), Error> {
        debug!("Creating resource set from parsed data.");

        RESOURCE_SET.with(|resource_set| {
            let mut set = resource_set.borrow_mut();
            set.images.clear();
            set.spritesheets.clear();
            set.fonts.clear();

            set.theme = Some(Rc::new(Theme::new("", builder_set.theme_builder)));

            for (id, sheet) in builder_set.spritesheet_builders {
                insert_if_ok_boxed("spritesheet", id, Spritesheet::new(sheet, &mut set),
                    &mut set.spritesheets);
            }

            for (id, font) in builder_set.font_builders {
                insert_if_ok_boxed("font", id, Font::new(font),
                &mut set.fonts);
            }

            if !set.fonts.contains_key(&Config::default_font()) {
                return invalid_data_error(&format!("Default font '{}' is not defined.",
                                                   Config::default_font()));
            }

            let empty = Rc::new(EmptyImage { });
            set.images.insert(empty.id(), empty);

            for (id, image) in builder_set.simple_builders {
                insert_if_ok_boxed("image", id, SimpleImage::new(image, &set), &mut set.images);
            }

            for (id, image) in builder_set.composed_builders {
                insert_if_ok_boxed("image", id, ComposedImage::new(image, &mut set), &mut set.images);
            }

            for (id, image) in builder_set.timer_builders {
                insert_if_ok_boxed("image", id, TimerImage::new(image, &set.images), &mut set.images);
            }

            for (id, image) in builder_set.animated_builders {
                insert_if_ok_boxed("image", id, AnimatedImage::new(image, &set.images), &mut set.images);
            }

            Ok(())
        })
    }

    pub fn get_image_else_empty(id: &str) -> Rc<Image> {
        RESOURCE_SET.with(|r| {
            match get_resource(id, &r.borrow().images) {
                None => {
                    warn!("No image with id '{}' found", id);
                    get_resource("empty", &r.borrow().images).unwrap()
                }, Some(ref image) => {
                    Rc::clone(image)
                }
            }
        })
    }

    pub fn get_empty_image() -> Rc<Image> {
        RESOURCE_SET.with(|r| get_resource("empty", &r.borrow().images)).unwrap()
    }

    pub fn get_default_font() -> Rc<Font> {
        RESOURCE_SET.with(|r| get_resource(&Config::default_font(), &r.borrow().fonts)).unwrap()
    }

    pub fn get_theme() -> Rc<Theme> {
        RESOURCE_SET.with(|r| Rc::clone(r.borrow().theme.as_ref().unwrap()))
    }

    pub fn get_spritesheet(id: &str) -> Option<Rc<Spritesheet>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().spritesheets))
    }

    pub fn get_sprite(id: &str) -> Result<Rc<Sprite>, Error> {
        RESOURCE_SET.with(|r| r.borrow().get_sprite_internal(id))
    }

    pub fn get_font(id: &str) -> Option<Rc<Font>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().fonts))
    }

    pub fn get_image(id: &str) -> Option<Rc<Image>> {
        RESOURCE_SET.with(|r| get_resource(id, &r.borrow().images))
    }

    /// Parses the `id` string to get a sprite from a spritesheet.  The string
    /// must be of the form {SPRITE_SHEET_ID}/{SPRITE_ID}
    pub fn get_sprite_internal(&self, id: &str) -> Result<Rc<Sprite>, Error> {
        let format_error = invalid_data_error("Image display must be \
                                              of format {SHEET_ID}/{SPRITE_ID}");

        let split_index = match id.find('/') {
            None => return format_error,
            Some(index) => index,
        };

        let (spritesheet_id, sprite_id) = id.split_at(split_index);
        if sprite_id.len() == 0 {
            return format_error;
        }
        let sprite_id = &sprite_id[1..];

        let sheet = match self.spritesheets.get(spritesheet_id) {
            None => return invalid_data_error(&format!("Unable to locate spritesheet '{}'",
                                                       spritesheet_id)),
            Some(sheet) => sheet,
        };

        let sprite = match sheet.sprites.get(sprite_id) {
            None => return invalid_data_error(
                &format!("Unable to locate sprite '{}' in spritesheet '{}'",
                         sprite_id, spritesheet_id)),
            Some(ref sprite) => Rc::clone(sprite),
        };

        Ok(sprite)
    }
}

pub fn all_resources<V: ?Sized>(map: &HashMap<String, Rc<V>>) -> Vec<Rc<V>> {
    map.iter().map(|ref res| Rc::clone(res.1)).collect()
}

pub fn get_resource<V: ?Sized>(id: &str, map: &HashMap<String, Rc<V>>) -> Option<Rc<V>> {
    let resource = map.get(id);

    match resource {
        None => None,
        Some(r) => Some(Rc::clone(r)),
    }
}

pub fn insert_if_ok<K: Eq + Hash + Display, V>(type_str: &str,
    key: K, val: Result<V, Error>, map: &mut HashMap<K, Rc<V>>) {

    trace!("Inserting resource of type {} with key {} into resource set.",
           type_str, key);
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => { (*map).insert(key, Rc::new(v)); }
    };
}

fn insert_if_ok_boxed<K: Eq + Hash + Display, V: ?Sized>(type_str: &str,
    key: K, val: Result<Rc<V>, Error>, map: &mut HashMap<K, Rc<V>>) {

    trace!("Inserting resource of type {} with key {} into resource set.",
           type_str, key);
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => { (*map).insert(key, Rc::clone(&v)); },
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

        if !entry.path().is_dir() { continue; }

        result.push(entry.path());
    }

    Ok(result)
}
