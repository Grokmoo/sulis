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

use std::fs;
use std::path::PathBuf;
use std::io::Error;
use std::time;

use chrono::prelude::*;

use sulis_core::{config, serde_yaml};
use sulis_core::resource::{ResourceBuilder, read_single_resource_path, write_to_file};
use sulis_core::util::{invalid_data_error};
use sulis_module::Module;
use {GameState, SaveState};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveFile {
    meta: SaveFileMetaData,
    state: SaveState,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveFileMetaData {
    pub player_name: String,
    pub datetime: String,
    pub current_area_name: String,

    #[serde(skip)]
    path: PathBuf,
}

fn get_save_dir() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("save");
    path.push(&Module::game().id);
    path
}

pub fn delete_save(save_file: &SaveFileMetaData) -> Result<(), Error> {
    let path = save_file.path.as_path();
    fs::remove_file(path)
}

pub fn load_state(save_file: &SaveFileMetaData) -> Result<(), Error> {
    let path = save_file.path.as_path();
    let save_file: SaveFile = read_single_resource_path(path)?;

    save_file.state.load();
    Ok(())
}

pub fn create_save() -> Result<(), Error> {
    let utc = Utc::now();
    let filename = format!("save_{}.yml", utc.format("%Y%m%d-%H%M%S%.3f"));

    let mut path = get_save_dir();
    path.push(filename);

    let meta = create_meta_data(utc.format("%c").to_string());

    let state = SaveState::create();

    let save = SaveFile {
        meta,
        state,
    };

    write_to_file(path.as_path(), &save)
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

        if extension != "yml" { continue; }

        let path_buf = path.to_path_buf();

        let save_file: SaveFile = match read_single_resource_path(&path_buf) {
            Ok(save_file) => save_file,
            Err(e) => {
                warn!("Unable to read save file: {}", path_buf.to_string_lossy());
                warn!("{}", e);
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
