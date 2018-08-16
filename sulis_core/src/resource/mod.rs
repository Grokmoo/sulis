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
pub use self::resource_builder_set::{read, read_single_resource, read_single_resource_path,
    read_to_string, write_to_file, write_json_to_file};

mod spritesheet;
pub use self::spritesheet::Spritesheet;
pub use self::spritesheet::Sprite;

mod font;
pub use self::font::Font;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Error;
use std::fmt::Display;
use std::hash::Hash;

use config::CONFIG;
use resource::resource_builder_set::ResourceBuilderSet;
use image::{Image, EmptyImage, SimpleImage, AnimatedImage, ComposedImage, TimerImage};
use util::invalid_data_error;
use ui::Theme;

thread_local! {
    static RESOURCE_SET: RefCell<ResourceSet> = RefCell::new(ResourceSet::new());
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn from_yaml(data: &str) -> Result<Self, Error>;
}

pub struct ResourceSet {
    pub (crate) theme: Option<Rc<Theme>>,
    pub (crate) images: HashMap<String, Rc<Image>>,
    pub (crate) spritesheets: HashMap<String, Rc<Spritesheet>>,
    pub (crate) fonts: HashMap<String, Rc<Font>>,
}

impl ResourceSet {
    pub fn new() -> ResourceSet {
        ResourceSet {
            theme: None,
            images: HashMap::new(),
            spritesheets: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn init(root_directory: &str) -> Result<(), Error> {
        let builder_set = ResourceBuilderSet::new(root_directory)?;

        debug!("Creating resource set from parsed data.");

        RESOURCE_SET.with(|resource_set| {
            let mut resource_set = resource_set.borrow_mut();

            resource_set.theme = Some(Rc::new(Theme::new("", builder_set.theme_builder)));

            let sheets_dir = &builder_set.spritesheets_dir;
            for (id, sheet) in builder_set.spritesheet_builders {
                insert_if_ok_boxed("spritesheet", id, Spritesheet::new(sheets_dir, sheet, &mut resource_set),
                    &mut resource_set.spritesheets);
            }

            let fonts_dir = &builder_set.fonts_dir;
            for (id, font) in builder_set.font_builders {
                insert_if_ok_boxed("font", id, Font::new(fonts_dir, font),
                &mut resource_set.fonts);
            }

            if !resource_set.fonts.contains_key(&CONFIG.display.default_font) {
                return invalid_data_error(&format!("Default font '{}' is not defined.",
                                                   CONFIG.display.default_font));
            }

            let empty = Rc::new(EmptyImage { });
            resource_set.images.insert(empty.id(), empty);

            for (id, image) in builder_set.simple_builders {
                insert_if_ok_boxed("image", id, SimpleImage::new(image, &resource_set),
                &mut resource_set.images);
            }

            for (id, image) in builder_set.composed_builders {
                insert_if_ok_boxed("image", id, ComposedImage::new(image,
                    &mut resource_set), &mut resource_set.images);
            }

            for (id, image) in builder_set.timer_builders {
                insert_if_ok_boxed("image", id, TimerImage::new(image,
                    &resource_set.images), &mut resource_set.images);
            }

            for (id, image) in builder_set.animated_builders {
                insert_if_ok_boxed("image", id, AnimatedImage::new(image,
                    &resource_set.images), &mut resource_set.images);
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
        RESOURCE_SET.with(|r| get_resource(&CONFIG.display.default_font, &r.borrow().fonts)).unwrap()
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
