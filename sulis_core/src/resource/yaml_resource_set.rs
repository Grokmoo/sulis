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
use std::path::{Path};

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

    Theme,
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
    Quest,
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
            "themes" => Theme,
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
            "quests" => Quest,
            "races" => Race,
            "sizes" => Size,
            "tiles" => Tile,
            "scripts" | "theme" => Skip,
            _ => return None,
        })
    }
}

impl YamlResourceSet {
    pub fn new(data_dir: &Path) -> Result<YamlResourceSet, Error> {
        let mut resources = HashMap::new();

        debug!("Parsing YAML in '{}'", data_dir.to_string_lossy().to_string());

        read_recursive(data_dir, data_dir,
            Some(YamlResourceKind::TopLevel), &mut resources);

        Ok(YamlResourceSet { resources })
    }

    pub fn append(&mut self, dir: &Path) {
        debug!("Appending resources in '{}'", dir.to_string_lossy().to_string());

        read_recursive(dir, dir, Some(YamlResourceKind::TopLevel),
            &mut self.resources);
    }
}

fn read_recursive(dir: &Path, top_level: &Path, kind: Option<YamlResourceKind>,
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

            read_recursive(&path, top_level, next_kind, resources);
        } else if path.is_file() {
            match kind {
                None => {
                    warn!("Skipping file '{:?}' as it is not in a recognized directory", path);
                },
                Some(kind) => {
                    read_file(&dir_str, &path, kind, resources);
                }
            }
        }
    }
}

fn read_file(dir_str: &str, path: &Path, kind: YamlResourceKind,
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

    let mut value: Value = match serde_yaml::from_str(&data) {
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
        merge_doc(dir_str, &path_str, entry, value);
        return;
    }

    match value {
        Value::Mapping(ref mut mapping) => {
            let dir = Value::String(dir_str.to_string());
            let mut seq = serde_yaml::Sequence::new();
            seq.push(dir);
            mapping.insert(Value::String(DIRECTORY_VAL_STR.to_string()), Value::Sequence(seq));

            let file = Value::String(path_str);
            let mut seq = serde_yaml::Sequence::new();
            seq.push(file);
            mapping.insert(Value::String(FILE_VAL_STR.to_string()), Value::Sequence(seq));
        },
        _ => warn!("Attempting to insert '{}' from '{}' which is not a mapping", id, path_str),
    }
    map.insert(id, value);
}

pub const DIRECTORY_VAL_STR: &str = "__directory__";
pub const FILE_VAL_STR: &str = "__file__";

fn merge_doc(dir: &str, name: &str, base: &mut Value, append: Value) {
    let directory_val = Value::String(DIRECTORY_VAL_STR.to_string());
    let file_val = Value::String(FILE_VAL_STR.to_string());

    match base {
        Value::Mapping(ref mut mapping) => {
            {
                let seq = mapping.get_mut(&directory_val).unwrap();
                if let Value::Sequence(ref mut seq) = seq {
                    seq.push(Value::String(name.to_string()));
                }
            }
            {
                let seq = mapping.get_mut(&file_val).unwrap();
                if let Value::Sequence(ref mut seq) = seq {
                    seq.push(Value::String(name.to_string()));
                }
            }

            match append {
                Value::Mapping(append) => merge_map(dir, name, mapping, append),
                _ => warn!("Unable to append '{}' to base YAML as it is not a mapping", name),
            }
        },
        _ => warn!("Unable to append '{}' to base YAML as it is not a mapping", name),
    }
}

fn merge_map(dir: &str, name: &str, map: &mut serde_yaml::Mapping,
             mut append: serde_yaml::Mapping) {
    let clear_base_keys: Value = Value::String("clear_base_keys".to_string());
    let remove_base_keys: Value = Value::String("remove_base_keys".to_string());

    if let Some(clear) = append.remove(&clear_base_keys) {
        match clear {
            Value::Bool(val) => {
                if val { map.clear(); }
            },
            _ => warn!("clear_base_keys must be a boolean in '{}'", name),
        }
    }

    if let Some(remove) = append.remove(&remove_base_keys) {
        match remove {
            Value::Sequence(seq) => {
                for value in seq {
                    map.remove(&value);
                }
            },
            _ => warn!("remove_base_keys must be a sequence of key-strings in '{}'", name),
        }
    }

    for (key, value) in append {
        if let Some(ref mut base) = map.get_mut(&key) {
            match base {
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
                Value::Sequence(ref mut seq) => {
                    match value {
                        Value::Sequence(append) => merge_sequence(dir, name, seq, append),
                        _ => warn!("Expected sequence for '{:?}' in '{}'", key, name),
                    }
                    continue;
                },
                Value::Mapping(ref mut map) => {
                    match value {
                        Value::Mapping(append) => merge_map(dir, name, map, append),
                        _ => warn!("Expected mapping for '{:?}' in '{}'", key, name),
                    }
                    continue;
                }
            }
        }

        map.insert(key, value);
    }
}

fn merge_sequence(_dir: &str, _name: &str, seq: &mut serde_yaml::Sequence,
                  append: serde_yaml::Sequence) {
    for value in append {
        seq.push(value);
    }
}
