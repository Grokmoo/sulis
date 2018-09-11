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
use std::io::Error;
use std::fs::{self};
use std::path::{Path, PathBuf};

use serde_yaml::{self, Value};

/// A set of resources that have been parsed into YAML values.  This is built up
/// by first reading the bottom level "data" layer, then the module layer, then
/// any active mods.  Each layer read is recursively merged into the previous,
/// adding new resources or keys to already existing resources.
pub struct YamlResourceSet {
    pub resources: HashMap<YamlResourceKind, HashMap<String, Value>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum YamlResourceKind {
    TopLevel,
    Skip,

    Font,
    AnimatedImage,
    ComposedImage,
    SimpleImage,
    TimerImage,
    Spritesheet,

    Ability,
    AbilityList,
    Actor,
    AiTemplate,
    Area,
    Class,
    Conversation,
    Cutscene,
    Encounter,
    Item,
    ItemAdjective,
    LootList,
    Prop,
    Race,
    Size,
    Tile,
}

impl YamlResourceKind {
    fn from_path(top_level: &Path, path: &Path) -> Option<YamlResourceKind> {
        let path_str = match path.strip_prefix(top_level) {
            Err(e) => {
                warn!("Unable to parse '{:?}' as subdir of '{:?}'", path, top_level);
                warn!("{}", e);
                return None;
            }, Ok(ref path) => path.to_string_lossy().to_string(),
        };

        YamlResourceKind::from_str(&path_str)
    }

    fn from_str(s: &str) -> Option<YamlResourceKind> {
        use self::YamlResourceKind::*;
        Some(match s {
            "fonts" => Font,
            "images/animated" | "images\\animated" => AnimatedImage,
            "images/composed" | "images\\composed" => ComposedImage,
            "images/simple" | "images\\simple"  => SimpleImage,
            "images/timer" | "images\\timer" => TimerImage,
            "spritesheets" => Spritesheet,

            "abilities" => Ability,
            "ability_lists" => AbilityList,
            "actors" => Actor,
            "ai" => AiTemplate,
            "areas" => Area,
            "classes" => Class,
            "conversations" => Conversation,
            "cutscenes" => Cutscene,
            "encounters" => Encounter,
            "items" => Item,
            "item_adjectives" => ItemAdjective,
            "loot_lists" => LootList,
            "props" => Prop,
            "races" => Race,
            "sizes" => Size,
            "tiles" => Tile,
            "scripts" | "theme" => Skip,
            _ => return None,
        })
    }
}

impl YamlResourceSet {
    pub fn new(data_dir: PathBuf) -> Result<YamlResourceSet, Error> {
        let mut resources = HashMap::new();

        debug!("Parsing YAML in '{}'", data_dir.to_string_lossy().to_string());

        let top_level = data_dir.clone();
        read_recursive(data_dir, &top_level.as_path(),
            Some(YamlResourceKind::TopLevel), &mut resources);

        Ok(YamlResourceSet { resources })
    }

    pub fn append(&mut self, dir: PathBuf) {
        debug!("Appending resources in '{}'", dir.to_string_lossy().to_string());

        let top_level = dir.clone();
        read_recursive(dir, &top_level.as_path(), Some(YamlResourceKind::TopLevel),
            &mut self.resources);
    }
}

fn read_recursive(dir: PathBuf, top_level: &Path, kind: Option<YamlResourceKind>,
                  resources: &mut HashMap<YamlResourceKind, HashMap<String, Value>>) {
    let dir_str = dir.to_string_lossy().to_string();
    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => {
            debug!("Unable to read directory: {}", dir_str);
            return;
        }
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading file: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            let next_kind = match kind {
                Some(YamlResourceKind::TopLevel) | None => {
                    let kind =  YamlResourceKind::from_path(top_level, &path);
                    if let Some(YamlResourceKind::Skip) = kind { continue; }
                    kind
                },
                Some(kind) => Some(kind),
            };

            read_recursive(path, top_level, next_kind, resources);
        } else if path.is_file() {
            match kind {
                None => {
                    warn!("Skipping file '{:?}' as it is not in a recognized directory", path);
                },
                Some(kind) => {
                    read_file(path, kind, resources);
                }
            }
        }
    }
}

fn read_file(path: PathBuf, kind: YamlResourceKind,
             resources: &mut HashMap<YamlResourceKind, HashMap<String, Value>>) {

    let path_str = path.to_string_lossy().to_string();

    if !path_str.ends_with("json") && ! path_str.ends_with("yml") { return; }

    debug!("Reading file as YAML at '{}'", path_str);
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            warn!("Error reading file at '{}': {}", path_str, e);
            return;
        }
    };

    let value: Value = match serde_yaml::from_str(&data) {
        Ok(value) => value,
        Err(e) => {
            warn!("Error parsing '{}' as YAML:", path_str);
            warn!("{}", e);
            return;
        }
    };

    let id = match value.get("id") {
        Some(ref id_value) => {
            match id_value {
                Value::String(ref s) => {
                    s.to_string()
                },
                _ => {
                    warn!("Top level ID is not a string in '{}'", path_str);
                    return;
                }
            }
        },
        None => {
            warn!("Unable to extract top level ID from '{}'", path_str);
            return;
        }
    };

    let map = resources.entry(kind).or_insert(HashMap::new());
    // use of entry API here seems to require us to clone our value since
    // we want to either append it or insert it
    //map.entry(id).and_modify(|entry| merge_doc(entry, value)).or_insert(value);
    if let Some(ref mut entry) = map.get_mut(&id) {
        merge_doc(&path_str, entry, value);
        return;
    }

    map.insert(id, value);
}

fn merge_doc(name: &str, base: &mut Value, append: Value) {
    match base {
        Value::Mapping(ref mut mapping) => {
            match append {
                Value::Mapping(append) => merge_map(name, mapping, append),
                _ => warn!("Unable to append '{}' to base YAML as it is not a mapping", name),
            }
        },
        _ => warn!("Unable to append '{}' to base YAML as it is not a mapping", name),
    }
}

fn merge_map(name: &str, map: &mut serde_yaml::Mapping, mut append: serde_yaml::Mapping) {
    let remove_base_keys = Value::String("remove_base_keys".to_string());
    if let Some(remove) = append.remove(&remove_base_keys) {
        match remove {
            Value::Sequence(seq) => {
                for value in seq {
                    map.remove(&value);
                }
            },
            _ => warn!("remove_base_keys must be a sequence of key-strings."),
        }
    }

    for (key, value) in append {
        if let Some(ref mut base) = map.get_mut(&key) {
            match base {
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
                Value::Sequence(ref mut seq) => {
                    match value {
                        Value::Sequence(append) => merge_sequence(name, seq, append),
                        _ => warn!("Expected sequence for '{:?}' in '{}'", key, name),
                    }
                    return;
                },
                Value::Mapping(ref mut map) => {
                    match value {
                        Value::Mapping(append) => merge_map(name, map, append),
                        _ => warn!("Expected mapping for '{:?}' in '{}'", key, name),
                    }
                    return;
                }
            }
        }

        map.insert(key, value);
    }
}

fn merge_sequence(_name: &str, seq: &mut serde_yaml::Sequence, append: serde_yaml::Sequence) {
    for value in append {
        seq.push(value);
    }
}
