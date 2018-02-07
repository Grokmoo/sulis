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

use std::io::{Read, Error, ErrorKind};
use std::path::Path;
use std::fs::{self, File};
use std::collections::HashMap;

use io::keyboard_event::Key;
use io::{KeyboardEvent, InputAction};
use util::Size;

use serde_yaml;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
  pub input: InputConfig,
  pub logging: LoggingConfig,
  pub editor: EditorConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditorConfig {
    pub module: String,
    pub transition_image: String,
    pub transition_size: Size,
    pub area: EditorAreaConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditorAreaConfig {
    pub filename: String,
    pub id: String,
    pub name: String,
    pub visibility_tile: String,
    pub layers: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub log_level: String,
    pub use_timestamps: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
    pub frame_rate: u32,
    pub animation_base_time_millis: u32,
    pub width: i32,
    pub height: i32,
    pub width_pixels: u32,
    pub height_pixels: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ResourcesConfig {
    pub directory: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InputConfig {
    pub keybindings: HashMap<Key, InputAction>
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub enum IOAdapter {
    Auto,
    Glium,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init("config.yml");
}

const CONFIG_BASE: &str = "config.sample.yml";

impl Config {
    fn init(filename: &str) -> Config {
        let config_path = Path::new(filename);
        let config_base_path = Path::new(CONFIG_BASE);

        if !config_path.is_file() {
            println!("{} not found, attempting to create it from {}", filename, CONFIG_BASE);
            match fs::copy(config_base_path, config_path) {
                Err(_) => {
                    let config_base_str = format!("../{}", CONFIG_BASE);
                    let config_base_path = Path::new(&config_base_str);
                    match fs::copy(config_base_path, config_path) {
                        Err(e) => {
                            eprintln!("{}", e);
                            eprintln!("Unable to create configuration file '{}'", filename);
                            eprintln!("Exiting...");
                            ::std::process::exit(1);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }

        }

        let config = Config::new(filename);
        match config {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("Fatal error loading the configuration from '{}'", filename);
                eprintln!("Exiting...");
                ::std::process::exit(1);
            }
        }
    }

    fn new(filename: &str) -> Result<Config, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;

        let config: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&file_data);
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                return Err(Error::new(ErrorKind::InvalidData, format!("{}", e)));
            }
        };

        match config.logging.log_level.as_ref() {
            "error" | "warn" | "info" | "debug" | "trace" => (),
            _ => return Err(Error::new(ErrorKind::InvalidData,
                    format!("log_level must be one of error, warn, info, debug, or trace")))
        };

        if config.display.width < 80 || config.display.height < 24 {
            return Err(Error::new(ErrorKind::InvalidData,
                "Minimum terminal display size is 80x24"));
        }

        Ok(config)
    }

    pub fn get_input_action(&self, k: Option<KeyboardEvent>) -> Option<InputAction> {
        match k {
            None => None,
            Some(k) => {
                debug!("Got keyboard input '{:?}'", k);
                match self.input.keybindings.get(&k.key) {
                    None => None,
                    Some(action) => Some(*action),
                }
            }
        }
    }
}
