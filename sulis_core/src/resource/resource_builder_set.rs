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
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

use serde;
use serde_json;
use serde_yaml;

use crate::image::animated_image::AnimatedImageBuilder;
use crate::image::composed_image::ComposedImageBuilder;
use crate::image::simple_image::SimpleImageBuilder;
use crate::image::timer_image::TimerImageBuilder;
use crate::image::window_image::WindowImageBuilder;
use crate::resource::font::FontBuilder;
use crate::resource::spritesheet::SpritesheetBuilder;
use crate::resource::*;
use crate::ui::ThemeBuilderSet;

#[derive(Debug)]
pub struct ResourceBuilderSet {
    pub theme_builder: ThemeBuilderSet,
    pub simple_builders: HashMap<String, SimpleImageBuilder>,
    pub composed_builders: HashMap<String, ComposedImageBuilder>,
    pub timer_builders: HashMap<String, TimerImageBuilder>,
    pub window_builders: HashMap<String, WindowImageBuilder>,
    pub animated_builders: HashMap<String, AnimatedImageBuilder>,
    pub spritesheet_builders: HashMap<String, SpritesheetBuilder>,
    pub font_builders: HashMap<String, FontBuilder>,
}

impl ResourceBuilderSet {
    pub fn from_yaml(resources: &mut YamlResourceSet) -> Result<ResourceBuilderSet, Error> {
        let theme_builders: HashMap<String, ThemeBuilderSet> = read_builders(resources, Theme)?;
        let mut themes_out = HashMap::new();
        for (_, theme_map) in theme_builders {
            for (id, theme) in theme_map.themes {
                if themes_out.contains_key(&id) {
                    warn!("Overwritting theme '{}'", id);
                }

                themes_out.insert(id, theme);
            }
        }

        use self::YamlResourceKind::*;
        Ok(ResourceBuilderSet {
            theme_builder: ThemeBuilderSet {
                id: "themes".to_string(),
                themes: themes_out,
            },
            font_builders: read_builders_insert_dirs(resources, Font)?,
            simple_builders: read_builders(resources, SimpleImage)?,
            composed_builders: read_builders(resources, ComposedImage)?,
            timer_builders: read_builders(resources, TimerImage)?,
            window_builders: read_builders(resources, WindowImage)?,
            animated_builders: read_builders(resources, AnimatedImage)?,
            spritesheet_builders: read_builders_insert_dirs(resources, Spritesheet)?,
        })
    }
}

pub fn read_builders<T: serde::de::DeserializeOwned>(
    resources: &mut YamlResourceSet,
    kind: YamlResourceKind,
) -> Result<HashMap<String, T>, Error> {
    read_builders_internal(resources, kind, false)
}

fn read_builders_insert_dirs<T: serde::de::DeserializeOwned>(
    resources: &mut YamlResourceSet,
    kind: YamlResourceKind,
) -> Result<HashMap<String, T>, Error> {
    read_builders_internal(resources, kind, true)
}

fn read_builders_internal<T: serde::de::DeserializeOwned>(
    resources: &mut YamlResourceSet,
    kind: YamlResourceKind,
    insert_dirs: bool,
) -> Result<HashMap<String, T>, Error> {
    let dir_key = serde_yaml::Value::String(yaml_resource_set::DIRECTORY_VAL_STR.to_string());
    let file_key = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());

    let mut builders = HashMap::new();
    for entries in resources.resources.remove(&kind).into_iter() {
        for (id, mut entry) in entries {
            let mut files = Vec::new();
            let mut dirs = serde_yaml::Sequence::new();

            if let serde_yaml::Value::Mapping(ref mut map) = entry {
                map.remove(&dir_key).into_iter().for_each(|val| {
                    if let serde_yaml::Value::Sequence(seq) = val {
                        dirs.extend(seq);
                    }
                });
                map.remove(&file_key).into_iter().for_each(|val| {
                    if let serde_yaml::Value::Sequence(seq) = val {
                        for file in seq {
                            if let serde_yaml::Value::String(file) = file {
                                files.push(file);
                            }
                        }
                    }
                });

                if insert_dirs {
                    map.insert(
                        serde_yaml::Value::String("source_dirs".to_string()),
                        serde_yaml::Value::Sequence(dirs),
                    );
                }
            }

            let builder: T = match read_builder_internal(entry) {
                Err(e) => {
                    warn!("Error in YAML file merged from {:?}", files);
                    return Err(e);
                }
                Ok(val) => val,
            };

            builders.insert(id, builder);
        }
    }
    Ok(builders)
}

pub fn read_builder<T: serde::de::DeserializeOwned>(
    mut value: serde_yaml::Value,
) -> Result<T, Error> {
    let dir_key = serde_yaml::Value::String(yaml_resource_set::DIRECTORY_VAL_STR.to_string());
    let file_key = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());

    if let serde_yaml::Value::Mapping(ref mut map) = value {
        map.remove(&dir_key);
        map.remove(&file_key);
    }

    read_builder_internal(value)
}

fn read_builder_internal<T: serde::de::DeserializeOwned>(
    value: serde_yaml::Value,
) -> Result<T, Error> {
    // we'd really rather not do this clone, but need to preserve the value in case there is an
    // error
    let value_clone = value.clone();
    let res: Result<T, serde_yaml::Error> = serde_yaml::from_value(value);

    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            let details = match handle_merged_error::<T>(value_clone) {
                Ok(details) => details,
                Err(handler_error) => {
                    warn!("Original Error: {}", e);
                    warn!("Handler Result: {}", handler_error);
                    String::new()
                }
            };

            Err(Error::new(ErrorKind::InvalidData, format!("{}", details)))
        }
    }
}

fn handle_merged_error<T: serde::de::DeserializeOwned>(
    value: serde_yaml::Value,
) -> Result<String, Error> {
    let value_string = serde_yaml::to_string(&value)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{}", e)))?;
    let result: Result<T, serde_yaml::Error> = serde_yaml::from_str(&value_string);

    match result {
        Ok(_) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("There was no error when parsing the value as a string"),
        )),
        Err(e) => Ok(e.to_string()),
    }
}

pub fn write_json_to_file<T: serde::ser::Serialize, P: AsRef<Path>>(
    filename: P,
    data: &T,
) -> Result<(), Error> {
    let file = File::create(filename)?;

    match serde_json::to_writer(file, data) {
        Err(e) => invalid_data_error(&format!("{}", e)),
        Ok(()) => Ok(()),
    }
}

pub fn write_to_file<T: serde::ser::Serialize, P: AsRef<Path>>(
    filename: P,
    data: &T,
) -> Result<(), Error> {
    let file = File::create(filename)?;

    match serde_yaml::to_writer(file, data) {
        Err(e) => invalid_data_error(&format!("{}", e)),
        Ok(()) => Ok(()),
    }
}

pub fn read_single_resource_path<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let data = fs::read_to_string(path)?;

    let result: Result<T, serde_yaml::Error> = serde_yaml::from_str(&data);
    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
    }
}

pub fn read_single_resource<T: serde::de::DeserializeOwned>(filename: &str) -> Result<T, Error> {
    let mut file = File::open(format!("{}.json", filename));
    if file.is_err() {
        file = File::open(format!("{}.yml", filename));
    }

    let mut file = match file {
        Err(_) => {
            return invalid_data_error(&format!(
                "Unable to locate '{}.json' or {}.yml'",
                filename, filename
            ));
        }
        Ok(f) => f,
    };

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let result: Result<T, serde_yaml::Error> = serde_yaml::from_str(&data);
    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
    }
}

pub fn read_to_string(root_dirs: &Vec<String>, dir: &str) -> HashMap<String, String> {
    let mut resources = HashMap::new();

    for root in root_dirs.iter() {
        read_recursive_to_string([root, dir].iter().collect(), &mut resources);
    }

    resources
}

fn read_recursive_to_string(dir: PathBuf, resources: &mut HashMap<String, String>) {
    let dir_str = dir.to_string_lossy().to_string();
    debug!("Reading resources from {}", dir_str);

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => {
            debug!("Unable to read directory: {}", dir_str);
            return;
        }
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(error) => {
                warn!("Error reading file: {}", error);
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            read_recursive_to_string(path, resources);
        } else if path.is_file() {
            read_file_to_string(path, resources);
        }
    }
}

fn read_file_to_string(path: PathBuf, resources: &mut HashMap<String, String>) {
    let path_str = path.to_string_lossy().to_string();

    // don't attempt to parse image files
    if !path_str.ends_with("lua") {
        return;
    }

    debug!("Reading file at {} to string", path_str);
    let data = match fs::read_to_string(path.clone()) {
        Ok(data) => data,
        Err(e) => {
            warn!("Error reading file at '{}': {}", path_str, e);
            return;
        }
    };

    let id: String = path
        .file_stem()
        .unwrap_or(OsStr::new(""))
        .to_string_lossy()
        .to_string();
    resources.insert(id, data);
}
