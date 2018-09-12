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

use std::env;
use std::cell::RefCell;
use std::io::{Read, Error, ErrorKind};
use std::path::Path;
use std::fs::{self, File};
use std::path::PathBuf;
use std::collections::HashMap;

use io::keyboard_event::Key;
use io::{KeyboardEvent, InputAction};

use serde_yaml;

thread_local! {
    static CONFIG: RefCell<Config> = RefCell::new(Config::init());
}

lazy_static! {
    pub static ref USER_DIR: PathBuf = get_user_dir();
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
  pub input: InputConfig,
  pub logging: LoggingConfig,
  pub editor: EditorConfig,

  #[serde(default)]
  pub debug: DebugConfig,
}

impl Config {
    pub fn set(config: Config) {
        CONFIG.with(|c| *c.borrow_mut() = config);
    }

    pub fn get_clone() -> Config {
        CONFIG.with(|c| c.borrow().clone())
    }

    pub fn display_resolution() -> (u32, u32) {
        CONFIG.with(|c| {
            let c = c.borrow();
            (c.display.width_pixels, c.display.height_pixels)
        })
    }

    pub fn monitor() -> usize {
        CONFIG.with(|c| c.borrow().display.monitor)
    }

    pub fn default_font() -> String {
        CONFIG.with(|c| c.borrow().display.default_font.to_string())
    }

    pub fn default_cursor() -> String {
        CONFIG.with(|c| c.borrow().display.default_cursor.to_string())
    }

    pub fn display_adapter() -> IOAdapter {
        CONFIG.with(|c| c.borrow().display.adapter)
    }

    pub fn display_mode() -> DisplayMode {
        CONFIG.with(|c| c.borrow().display.mode)
    }

    pub fn ui_height() -> i32 {
        CONFIG.with(|c| c.borrow().display.height)
    }

    pub fn ui_width() -> i32 {
        CONFIG.with(|c| c.borrow().display.width)
    }

    pub fn ui_size() -> (i32, i32) {
        CONFIG.with(|c| {
            let c = c.borrow();
            (c.display.width, c.display.height)
        })
    }

    pub fn frame_rate() -> u32 {
        CONFIG.with(|c| c.borrow().display.frame_rate)
    }

    pub fn animation_base_time_millis() -> u32 {
        CONFIG.with(|c| c.borrow().display.animation_base_time_millis)
    }

    pub fn logging_config() -> LoggingConfig {
        CONFIG.with(|c| c.borrow().logging.clone())
    }

    pub fn debug() -> DebugConfig {
        CONFIG.with(|c| c.borrow().debug.clone())
    }

    pub fn editor_config() -> EditorConfig {
        CONFIG.with(|c| c.borrow().editor.clone())
    }

    pub fn resources_config() -> ResourcesConfig {
        CONFIG.with(|c| c.borrow().resources.clone())
    }

    pub fn get_input_action(k: KeyboardEvent) -> Option<InputAction> {
        debug!("Got keyboard input '{:?}'", k);
        CONFIG.with(|c| {
            match c.borrow().input.keybindings.get(&k.key) {
                None => None,
                Some(action) => Some(*action),
            }
        })
    }

    pub fn scroll_speed() -> f32 {
        CONFIG.with(|c| c.borrow().input.scroll_speed)
    }

    pub fn edge_scrolling() -> bool {
        CONFIG.with(|c| c.borrow().input.edge_scrolling)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub encounter_spawning: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            encounter_spawning: true
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditorConfig {
    pub module: String,
    pub cursor: String,
    pub transition_image: String,
    pub transition_size: String,
    pub area: EditorAreaConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditorAreaConfig {
    pub filename: String,
    pub id: String,
    pub name: String,
    pub visibility_tile: String,
    pub explored_tile: String,
    pub encounter_tile: String,
    pub layers: Vec<String>,
    pub elev_tiles: Vec<String>,
    pub entity_layer: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub log_level: String,
    pub use_timestamps: bool,
    pub append: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
    pub mode: DisplayMode,
    pub monitor: usize,
    pub frame_rate: u32,
    pub animation_base_time_millis: u32,
    pub width: i32,
    pub height: i32,
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub default_font: String,
    pub default_cursor: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub enum DisplayMode {
    Window,
    BorderlessWindow,
    Fullscreen,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ResourcesConfig {
    pub directory: String,
    pub campaigns_directory: String,
    pub mods_directory: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InputConfig {
    pub edge_scrolling: bool,
    pub scroll_speed: f32,
    pub keybindings: HashMap<Key, InputAction>
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub enum IOAdapter {
    Auto,
    Glium,
}

#[cfg(not(target_os = "windows"))]
fn get_user_dir() -> PathBuf {
    let mut path = match env::var("XDG_CONFIG_HOME") {
        Ok(path_str) => {
            PathBuf::from(path_str)
        },
        Err(_) => {
            let mut path = get_home_dir();
            path.push(".config/");
            path
        }
    };
    path.push("sulis/");
    path
}

#[cfg(target_os = "windows")]
fn get_user_dir() -> PathBuf {
    let mut path = get_home_dir();
    path.push("My Documents");
    path.push("My Games");
    path.push("Sulis");
    path
}

fn get_home_dir() -> PathBuf {
    match env::home_dir() {
        Some(path) => path,
        None => PathBuf::new(),
    }
}

const CONFIG_FILENAME: &str = "config.yml";
pub const CONFIG_BASE: &str = "config.sample.yml";

pub fn create_dir_and_warn(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        warn!("Unable to create dir: '{:?}'", path);
        warn!("{}", e);
    }
}

impl Config {
    fn init() -> Config {
        let mut config_path = USER_DIR.clone();
        config_path.push(CONFIG_FILENAME);
        let config_path = config_path.as_path();

        if !config_path.is_file() {
            Config::create_config_from_sample(config_path);
        }

        match Config::new(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("Error parsing config file at '{}', attempting delete.", CONFIG_FILENAME);

                Config::create_config_from_sample(config_path);

                match Config::new(config_path) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("{}", e);
                        eprintln!("Fatal error in sample config.  Exiting...");
                        ::std::process::exit(1);
                    }
                }
            }
        }
    }

    fn create_config_from_sample(config_path: &Path) {
        let config_base_path = Path::new(CONFIG_BASE);

        println!("{} not found, attempting to create it from {}", CONFIG_FILENAME, CONFIG_BASE);
        if let Some(path) = config_path.parent() {
            create_dir_and_warn(path);
        }

        match fs::copy(config_base_path, config_path) {
            Err(_) => {
                let config_base_str = format!("../{}", CONFIG_BASE);
                let config_base_path = Path::new(&config_base_str);
                match fs::copy(config_base_path, config_path) {
                    Err(e) => {
                        eprintln!("{}", e);
                        eprintln!("Unable to create configuration file '{}'", CONFIG_FILENAME);
                        eprintln!("Exiting...");
                        ::std::process::exit(1);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub fn new(filepath: &Path) -> Result<Config, Error> {
        let mut f = File::open(filepath)?;
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
}
