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

use std::path::{PathBuf};
use std::fs;
use std::io::Error;

use sulis_core::resource::{ResourceBuilder, read_single_resource};
use sulis_core::util::{invalid_data_error};
use sulis_core::serde_yaml;

pub fn get_available_modifications(root_dir: &str) -> Vec<ModificationInfo> {
    let mut mods = Vec::new();

    let path = PathBuf::from(root_dir);

    let dir_entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => {
            warn!("Unable to read directory: {}", root_dir);
            return mods;
        }
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Error reading entry: {}", e);
                continue;
            }
        };

        if entry.path().is_dir() {
            let modi = match ModificationInfo::from_dir(entry.path()) {
                Ok(modi) => modi,
                Err(e) => {
                    warn!("Unable to read modification from '{:?}'", entry.path());
                    warn!("{}", e);
                    continue;
                }
            };

            mods.push(modi);
        }
    }

    mods
}

pub struct ModificationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl ModificationInfo {
    pub fn from_dir(path: PathBuf) -> Result<ModificationInfo, Error> {
        let path_str = path.to_string_lossy().to_string();
        let builder = read_single_resource(&format!("{}/mod", path_str))?;

        Ok(ModificationInfo::new(builder)?)
    }

    pub fn new(builder: ModificationInfoBuilder) -> Result<ModificationInfo, Error> {
        Ok(ModificationInfo {
            name: builder.name,
            description: builder.description,
            id: builder.id,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ModificationInfoBuilder {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl ResourceBuilder for ModificationInfoBuilder {
    fn owned_id(&self) -> String {
        "ModificationInfoBuilder".to_string()
    }

    fn from_yaml(data: &str) -> Result<ModificationInfoBuilder, Error> {
        let resource: Result<ModificationInfoBuilder, serde_yaml::Error> =
            serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(e) => invalid_data_error(&format!("{}", e)),
        }
    }
}
