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

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Error};
use std::time;

use chrono::prelude::*;

use sulis_core::{config, serde_yaml, util::self, serde_json};
use sulis_core::resource::{ResourceBuilder, read_single_resource_path, write_json_to_file};
use sulis_core::util::{invalid_data_error};
use sulis_module::Module;
use {GameState, SaveState};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveFile {
    meta: SaveFileMetaData,
    state: SaveState,
}

impl SaveFile {
    fn from_json(data: &str) -> Result<Self, Error> {
        let resource: Result<SaveFile, serde_json::Error> = serde_json::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveFileMetaData {
    pub player_name: String,
    pub datetime: String,
    pub current_area_name: String,

    #[serde(skip)]
    path: PathBuf,

    #[serde(skip)]
    pub error: Option<String>,
}

fn get_save_dir() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("save");
    path.push(&Module::campaign().id);
    path
}

pub fn delete_save(save_file: &SaveFileMetaData) -> Result<(), Error> {
    let path = save_file.path.as_path();
    fs::remove_file(path)
}

pub fn load_state(save_file: &SaveFileMetaData) -> Result<SaveState, Error> {
    let path = save_file.path.as_path();
    let save_file: SaveFile = read_single_resource_path(path)?;

    Ok(save_file.state)
}

pub fn create_save() -> Result<(), Error> {
    let start_time = time::Instant::now();
    info!("Start save");

    let utc = Utc::now();
    let filename = format!("save_{}.json", utc.format("%Y%m%d-%H%M%S%.3f"));

    let mut path = get_save_dir();
    if !path.is_dir() {
        trace!("Save dir '{:?}' not found, attempting to create it.", path);
        fs::create_dir_all(path.clone())?;
    }

    path.push(filename);

    let meta = create_meta_data(utc.format("%c").to_string());

    info!("  Filename and meta data creation complete in {} secs",
          util::format_elapsed_secs(start_time.elapsed()));

    let state = SaveState::create();

    let save = SaveFile {
        meta,
        state,
    };

    info!("  Save data created in {} secs", util::format_elapsed_secs(start_time.elapsed()));

    let result = write_json_to_file(path.as_path(), &save);

    info!("  Save to disk complete in {} secs", util::format_elapsed_secs(start_time.elapsed()));

    result
}

fn create_meta_data(datetime: String) -> SaveFileMetaData {
    let cur_area = GameState::area_state();
    let cur_area = cur_area.borrow();
    let player = GameState::player();
    let player = player.borrow();

    SaveFileMetaData {
        player_name: player.actor.actor.name.to_string(),
        datetime,
        current_area_name: cur_area.area.name.to_string(),
        path: Default::default(),
        error: None,
    }
}

pub fn has_available_save_files() -> bool {
    let dir = get_save_dir();
    if !dir.is_dir() { return false; }

    let dir_entries = match fs::read_dir(dir) {
        Err(_) => return false,
        Ok(entries) => entries,
    };

    for entry in dir_entries {
        let entry = match entry {
            Err(_) => continue,
            Ok(entry) => entry,
        };

        let path = entry.path();
        if !path.is_file() { continue; }

        let extension = match path.extension() {
            None => continue,
            Some(ext) => ext.to_string_lossy(),
        };

        if extension != "json" { continue; }

        return true;
    }

    false
}

fn read_save_file(path: &Path) -> Result<SaveFile, Error> {
    let mut file = File::open(path)?;

    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;

    SaveFile::from_json(&file_data)
}

fn create_error_meta(path: PathBuf, error: Error) -> SaveFileMetaData {
    let time = match fs::metadata(&path) {
        Err(e) => {
            warn!("Unable to get metadata for invalid save file at {:?}", path);
            warn!("{}", e);
            Utc::now()
        },
        Ok(meta) => {
            match meta.created() {
                Err(e) => {
                    warn!("Unable to get creation time for invalid save file at {:?}", path);
                    warn!("{}", e);
                    Utc::now()
                },
                Ok(time) => DateTime::from(time)
            }
        }
    };

    let datetime = time.format("%c").to_string();

    SaveFileMetaData {
        player_name: "Unknown Player".to_string(),
        datetime,
        current_area_name: "Unknown Area".to_string(),
        path,
        error: Some(error.to_string()),
    }
}

pub fn get_available_save_files() -> Result<Vec<SaveFileMetaData>, Error> {
    let mut results = Vec::new();

    let dir = get_save_dir();
    debug!("Reading save games from {}", dir.to_string_lossy());

    if !dir.is_dir() {
        fs::create_dir_all(dir.clone())?;
    }

    let dir_entries = fs::read_dir(dir)?;

    for entry in dir_entries {
        trace!("Checking entry {:?}", entry);
        let entry = entry?;

        let path = entry.path();
        if !path.is_file() { continue; }

        let extension = match path.extension() {
            None => continue,
            Some(ext) => ext.to_string_lossy(),
        };

        if extension != "json" { continue; }

        let path_buf = path.to_path_buf();

        let save_file: SaveFile = match read_save_file(&path_buf) {
            Ok(save_file) => save_file,
            Err(e) => {
                warn!("Unable to read save file: {}", path_buf.to_string_lossy());
                warn!("{}", e);
                results.push(create_error_meta(path_buf, e));
                continue;
            }
        };

        let mut meta = save_file.meta;
        meta.path = path_buf;

        results.push(meta);
    }

    results.sort_by(|f1, f2| {
        let t1 = time_modified(f1);
        let t2 = time_modified(f2);

        t2.cmp(&t1)
    });

    Ok(results)
}

fn time_modified(data: &SaveFileMetaData) -> time::SystemTime {
    let metadata = fs::metadata(data.path.as_path());

    match metadata {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(time) => time,
                Err(_) => time::UNIX_EPOCH,
            }
        },
        Err(_) => time::UNIX_EPOCH,
    }
}

impl ResourceBuilder for SaveFile {
    fn owned_id(&self) -> String {
        self.meta.player_name.to_string()
    }

    fn from_yaml(data: &str) -> Result<Self, Error> {
        let resource: Result<SaveFile, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
