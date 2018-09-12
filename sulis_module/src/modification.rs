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

use std::fmt::{self, Display};
use std::path::{PathBuf};
use std::io::Error;

use sulis_core::config::{self, Config};
use sulis_core::resource::{subdirs, read_single_resource};

pub fn get_available_modifications() -> Vec<ModificationInfo> {
    let root_dir = Config::resources_config().mods_directory;
    let mut user_dir = config::USER_DIR.clone();
    user_dir.push(&root_dir);

    let mut mods = Vec::new();

    let mut dirs = Vec::new();
    match subdirs(&root_dir) {
        Ok(mut subdirs) => dirs.append(&mut subdirs),
        Err(e) => warn!("Unable to read mods from '{}': {}", root_dir, e),
    }

    match subdirs(&user_dir) {
        Ok(mut subdirs) => dirs.append(&mut subdirs),
        Err(e) => warn!("Unable to read mods from '{:?}': {}", user_dir, e),
    }

    for dir in dirs {
        match ModificationInfo::from_dir(dir.clone()) {
            Ok(modi) => mods.push(modi),
            Err(e) => warn!("Error reading module from '{:?}': {}", dir, e),
        }
    }

    mods
}

#[derive(Debug, Clone)]
pub struct ModificationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dir: String,
}

impl Display for ModificationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ModificationInfo {
    pub fn from_dir(path: PathBuf) -> Result<ModificationInfo, Error> {
        let path_str = path.to_string_lossy().to_string();
        let builder: ModificationInfoBuilder = read_single_resource(&format!("{}/mod", path_str))?;

        Ok(ModificationInfo {
            name: builder.name,
            description: builder.description,
            id: builder.id,
            dir: path_str,
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
