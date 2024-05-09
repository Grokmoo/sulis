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

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use std::path::PathBuf;

use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};
use log::{Level, LevelFilter};

use crate::io::keyboard_event::Key;
use crate::io::{event::ClickKind, InputActionKind, InputAction, KeyboardEvent};

thread_local! {
    static CONFIG: RefCell<Config> = RefCell::new(Config::init());
    static OLD_CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) };
}

lazy_static! {
    pub static ref USER_DIR: PathBuf = get_user_dir();
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub revision: u32,
    pub display: DisplayConfig,
    pub audio: AudioConfig,
    pub resources: ResourcesConfig,
    pub input: InputConfig,
    pub logging: LoggingConfig,
    pub editor: EditorConfig,

    #[serde(default)]
    pub debug: DebugConfig,
}

impl Config {
    pub fn set(config: Config) {
        let old_config = CONFIG.with(|c| c.replace(config));

        OLD_CONFIG.with(|c| c.replace(Some(old_config)));
    }

    pub fn take_old_config() -> Option<Config> {
        OLD_CONFIG.with(|c| c.replace(None))
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

    pub fn vsync_enabled() -> bool {
        CONFIG.with(|c| c.borrow().display.vsync_enabled)
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

    pub fn default_zoom() -> f32 {
        CONFIG.with(|c| c.borrow().display.default_zoom)
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

    pub fn audio_config() -> AudioConfig {
        CONFIG.with(|c| c.borrow().audio.clone())
    }

    pub fn editor_config() -> EditorConfig {
        CONFIG.with(|c| c.borrow().editor.clone())
    }

    pub fn resources_config() -> ResourcesConfig {
        CONFIG.with(|c| c.borrow().resources.clone())
    }

    pub fn get_keybindings() -> HashMap<InputActionKind, Key> {
        CONFIG.with(|c| {
            c.borrow()
                .input
                .keybindings
                .iter()
                .map(|(k, v)| (*v, *k))
                .collect()
        })
    }

    pub fn get_click_action(button: RawClick) -> ClickKind {
        CONFIG.with(|c| *c.borrow().input.click_actions.get(&button).unwrap())
    }

    pub fn get_input_action(k: KeyboardEvent) -> Option<InputAction> {
        debug!("Got keyboard input '{:?}'", k);
        CONFIG.with(|c| {
            let kind = c.borrow().input.keybindings.get(&k.key).copied();

            kind.map(|kind| InputAction { kind, state: k.state })
        })
    }

    pub fn scroll_speed() -> f32 {
        CONFIG.with(|c| c.borrow().input.scroll_speed)
    }

    pub fn edge_scrolling() -> bool {
        CONFIG.with(|c| c.borrow().input.edge_scrolling)
    }

    pub fn crit_screen_shake() -> bool {
        CONFIG.with(|c| c.borrow().input.crit_screen_shake)
    }

    pub fn scroll_to_active() -> bool {
        CONFIG.with(|c| c.borrow().display.scroll_to_active)
    }

    pub fn bench_log_level() -> Level {
        CONFIG.with(|c| c.borrow().logging.bench_log_level)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub encounter_spawning: bool,
    pub limit_line_of_sight: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            encounter_spawning: true,
            limit_line_of_sight: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditorConfig {
    pub module: String,
    pub cursor: String,
    pub transition_image: String,

    #[serde(deserialize_with = "de_non_empty_vec")]
    pub transition_sizes: Vec<String>,
    pub area: EditorAreaConfig,
}

fn de_non_empty_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = Vec::deserialize(deserializer)?;

    if result.is_empty() {
        use serde::de::Error;
        return Err(Error::custom("Vec must be non-empty"));
    }

    Ok(result)
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

    #[serde(deserialize_with = "de_non_empty_vec")]
    pub layers: Vec<String>,

    #[serde(deserialize_with = "de_non_empty_vec")]
    pub elev_tiles: Vec<String>,
    pub entity_layer: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub log_level: LevelFilter,
    pub stderr_log_level: LevelFilter,
    pub bench_log_level: Level,
    pub use_timestamps: bool,
    pub append: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    pub device: usize,
    pub master_volume: f32,
    pub music_volume: f32,
    pub effects_volume: f32,
    pub ambient_volume: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DisplayConfig {
    pub mode: DisplayMode,
    pub monitor: usize,
    pub frame_rate: u32,
    pub animation_base_time_millis: u32,
    pub default_zoom: f32,
    pub width: i32,
    pub height: i32,
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub default_font: String,
    pub default_cursor: String,
    pub scroll_to_active: bool,
    pub vsync_enabled: bool,
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
    pub keybindings: HashMap<Key, InputActionKind>,
    pub click_actions: HashMap<RawClick, ClickKind>,
    pub crit_screen_shake: bool,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub enum RawClick {
    Left,
    Right,
    Middle,
}

const RAW_CLICKS: [RawClick; 3] = [RawClick::Left, RawClick::Right, RawClick::Middle];

impl RawClick {
    pub fn iter() -> impl Iterator<Item = &'static RawClick> {
        RAW_CLICKS.iter()
    }
}

#[cfg(not(target_os = "windows"))]
fn get_user_dir() -> PathBuf {
    let mut path = match ::std::env::var("XDG_CONFIG_HOME") {
        Ok(path_str) => PathBuf::from(path_str),
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
    match home::home_dir() {
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
        let revision = match Config::new(Path::new(CONFIG_BASE), 0) {
            Ok(config) => config.revision,
            Err(orig_e) => match Config::new(&Path::new("../").join(CONFIG_BASE), 0) {
                Err(_) => {
                    eprintln!("{orig_e}");
                    eprintln!("Unable to parse revision from config.sample");
                    std::process::exit(1);
                }
                Ok(config) => config.revision,
            },
        };

        let mut config_path = USER_DIR.clone();
        config_path.push(CONFIG_FILENAME);
        let config_path = config_path.as_path();

        if !config_path.is_file() {
            Config::create_config_from_sample(config_path);
        }

        match Config::new(config_path, revision) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{e}");
                eprintln!(
                    "Error parsing config file at '{CONFIG_FILENAME}', attempting delete."
                );

                Config::create_config_from_sample(config_path);

                match Config::new(config_path, revision) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("{e}");
                        eprintln!("Fatal error in sample config.  Exiting...");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    fn create_config_from_sample(config_path: &Path) {
        let config_base_path = Path::new(CONFIG_BASE);

        println!(
            "{CONFIG_FILENAME} not found, attempting to create it from {CONFIG_BASE}"
        );
        if let Some(path) = config_path.parent() {
            create_dir_and_warn(path);
        }

        if fs::copy(config_base_path, config_path).is_err() {
            let config_base_str = format!("../{CONFIG_BASE}");
            let config_base_path = Path::new(&config_base_str);
            if let Err(e) = fs::copy(config_base_path, config_path) {
                eprintln!("{e}");
                eprintln!("Unable to create configuration file '{CONFIG_FILENAME}'");
                eprintln!("Exiting...");
                ::std::process::exit(1);
            }
        }
    }

    pub fn new(filepath: &Path, required_revision: u32) -> Result<Config, Error> {
        let mut f = File::open(filepath)?;

        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;

        let config: Result<Config, serde_yaml::Error> = serde_yaml::from_str(&file_data);
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                return Err(Error::new(ErrorKind::InvalidData, format!("{e}")));
            }
        };

        for key in RawClick::iter() {
            if !config.input.click_actions.contains_key(key) {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Must specify an action for each of Left, Right & Middle Click",
                ));
            }
        }

        if config.revision < required_revision {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Config has old revision: {}", config.revision),
            ));
        }

        Ok(config)
    }
}
