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
use std::fs::{self, File};
use std::io::{Read, Error};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde;
use serde_yaml;
use serde_json;

use resource::*;
use resource::spritesheet::SpritesheetBuilder;
use resource::font::FontBuilder;
use image::simple_image::SimpleImageBuilder;
use image::composed_image::ComposedImageBuilder;
use image::timer_image::TimerImageBuilder;
use image::animated_image::AnimatedImageBuilder;
use ui::theme::{ThemeBuilder, create_theme};

#[derive(Debug)]
pub struct ResourceBuilderSet {
    pub theme_builder: ThemeBuilder,
    pub simple_builders: HashMap<String, SimpleImageBuilder>,
    pub composed_builders: HashMap<String, ComposedImageBuilder>,
    pub timer_builders: HashMap<String, TimerImageBuilder>,
    pub animated_builders: HashMap<String, AnimatedImageBuilder>,
    pub spritesheet_builders: HashMap<String, SpritesheetBuilder>,
    pub font_builders: HashMap<String, FontBuilder>,
}

impl ResourceBuilderSet {
    pub fn from_yaml(resources: &mut YamlResourceSet,
                     theme_dir: &str) -> Result<ResourceBuilderSet, Error> {
        let theme_builder = build_theme(theme_dir)?;

        use self::YamlResourceKind::*;
        Ok(ResourceBuilderSet {
            theme_builder,
            font_builders: read_builders_insert_dirs(resources, Font)?,
            simple_builders: read_builders(resources, SimpleImage)?,
            composed_builders: read_builders(resources, ComposedImage)?,
            timer_builders: read_builders(resources, TimerImage)?,
            animated_builders: read_builders(resources, AnimatedImage)?,
            spritesheet_builders: read_builders_insert_dirs(resources, Spritesheet)?,
        })
    }
}

pub fn read_builders<T: serde::de::DeserializeOwned>(resources: &mut YamlResourceSet,
    kind: YamlResourceKind) -> Result<HashMap<String, T>, Error> {

    read_builders_internal(resources, kind, false)
}

fn read_builders_insert_dirs<T: serde::de::DeserializeOwned>(resources: &mut YamlResourceSet,
    kind: YamlResourceKind) -> Result<HashMap<String, T>, Error> {

    read_builders_internal(resources, kind, true)
}

fn read_builders_internal<T: serde::de::DeserializeOwned>(resources: &mut YamlResourceSet,
    kind: YamlResourceKind, insert_dirs: bool) -> Result<HashMap<String, T>, Error> {

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
                    map.insert(serde_yaml::Value::String("source_dirs".to_string()),
                        serde_yaml::Value::Sequence(dirs));
                }
            }

            let builder: T = match read_builder_internal(entry) {
                Err(e) => {
                    warn!("Error in YAML file merged from {:?}", files);
                    return Err(e);
                }, Ok(val) => val,
            };

            builders.insert(id, builder);
        }
    }
    Ok(builders)
}

pub fn read_builder<T: serde::de::DeserializeOwned>(mut value: serde_yaml::Value) -> Result<T, Error> {
    let dir_key = serde_yaml::Value::String(yaml_resource_set::DIRECTORY_VAL_STR.to_string());
    let file_key = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());

    if let serde_yaml::Value::Mapping(ref mut map) = value {
        map.remove(&dir_key);
        map.remove(&file_key);
    }

    read_builder_internal(value)
}

fn read_builder_internal<T: serde::de::DeserializeOwned>(value: serde_yaml::Value) -> Result<T, Error> {
    let res: Result<T, serde_yaml::Error> = serde_yaml::from_value(value);

    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
    }
}

fn build_theme(theme_dir: &str) -> Result<ThemeBuilder, Error> {
    info!("Reading theme from '{}'", theme_dir);

    let mut theme_builder = create_theme(theme_dir, "theme")?;

    theme_builder.expand_references()?;

    Ok(theme_builder)
}

pub fn write_json_to_file<T: serde::ser::Serialize,
    P: AsRef<Path>>(filename: P, data: &T) -> Result<(), Error> {

    let file = File::create(filename)?;

    match serde_json::to_writer(file, data) {
        Err(e) => invalid_data_error(&format!("{}", e)),
        Ok(()) => Ok(()),
    }
}

pub fn write_to_file<T: serde::ser::Serialize, P: AsRef<Path>>(filename: P, data: &T) -> Result<(), Error> {
    let file = File::create(filename)?;

    match serde_yaml::to_writer(file, data) {
        Err(e) => invalid_data_error(&format!("{}", e)),
        Ok(()) => Ok(()),
    }
}

pub fn read_single_resource_path<T: ResourceBuilder>(path: &Path) -> Result<T, Error> {
    let mut file = File::open(path)?;

    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;

    T::from_yaml(&file_data)
}

pub fn read_single_resource<T: ResourceBuilder>(filename: &str) -> Result<T, Error> {
    let mut file = File::open(format!("{}.json", filename));
    if file.is_err() {
        file = File::open(format!("{}.yml", filename));
    }

    if file.is_err() {
        return invalid_data_error(
            &format!("Unable to locate '{}.json' or '{}.yml'", filename, filename));
    }

    let mut file_data = String::new();
    file.unwrap().read_to_string(&mut file_data)?;

    T::from_yaml(&file_data)
}

pub fn read<T: ResourceBuilder>(root_dirs: &Vec<&str>, dir: &str) -> HashMap<String, T> {
    let mut resources: HashMap<String, T> = HashMap::new();

    for root in root_dirs.iter() {
        read_recursive([root, dir].iter().collect(), &mut resources, &load_resource_builder);
    }

    if resources.is_empty() {
        info!("Unable to read any resources from subdir '{}' in directories: '{:?}'", dir, root_dirs);
    }

    resources
}

pub fn read_to_string(root_dirs: &Vec<String>, dir: &str) -> HashMap<String, String> {
    let mut resources = HashMap::new();

    for root in root_dirs.iter() {
        read_recursive([root, dir].iter().collect(), &mut resources, &load_resource_to_string);
    }

    if resources.is_empty() {
        info!("Unable to read any resources from subdir '{}' in directories: '{:?}'", dir, root_dirs);
    }

    resources
}

fn read_recursive<T>(dir: PathBuf, resources: &mut HashMap<String, T>,
                                      func: &Fn(&str, String, &mut HashMap<String, T>)) {
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
        trace!("Found entry {:?}", entry);
        let entry = match entry {
            Ok(e) => e,
            Err(error) => {
                warn!("Error reading file: {}", error);
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            read_recursive(path, resources, func);
        } else if path.is_file() {
            read_file(path, resources, func);
        }
    }
}

fn read_file<T>(path: PathBuf, resources: &mut HashMap<String, T>,
                                 func: &Fn(&str, String, &mut HashMap<String, T>)) {
    let path_str = path.to_string_lossy().to_string();

    // don't attempt to parse image files
    if !path_str.ends_with("json") && !path_str.ends_with("yml") && !path_str.ends_with("lua") {
        trace!("Skipping file '{}' because of unrecognized extension", path_str);
        return
    }

    debug!("Reading file at {}", path_str);
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            warn!("Error reading file: {}", error);
            return;
        }
    };

    let mut file_data = String::new();
    if file.read_to_string(&mut file_data).is_err() {
        warn!("Error reading file data from file {}", path_str);
        return;
    }
    trace!("Read file data.");

    (func)(&path_str, file_data, resources);
}

fn load_resource_to_string(path: &str, file_data: String,
                           resources: &mut HashMap<String, String>) {
    let path_buf = PathBuf::from(path);

    let id: String = path_buf.file_stem().unwrap_or(OsStr::new("")).to_string_lossy().to_string();

    trace!("Created string resource '{}'", id);
    if resources.contains_key(&id) {
        debug!("Overwriting resource with key: {} in {}", id, path);
    }

    resources.insert(id, file_data);
}

fn load_resource_builder<T: ResourceBuilder>(path: &str, file_data: String,
                                             resources: &mut HashMap<String, T>) {
    let resource = match T::from_yaml(&file_data) {
        Ok(res) => res,
        Err(error) => {
            warn!("Error parsing file data: {:?}", path);
            warn!("  {}", error);
            return;
        }
    };

    let id = resource.owned_id();

    trace!("Created resource '{}'", id);
    if resources.contains_key(&id) {
        debug!("Overwriting resource with key: {} in {}", id, path);
    }

    resources.insert(id, resource);
}
