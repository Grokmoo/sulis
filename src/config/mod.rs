use std::io::{Read, Error, ErrorKind};
use std::fs::File;
use std::collections::HashMap;

use io::keyboard_event::Key;
use io::{KeyboardEvent, InputAction};

use serde_json;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
  pub input: InputConfig,
  pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
    pub frame_rate: u32,
    pub animation_base_time_millis: u32,
    pub width: i32,
    pub height: i32,
    pub cursor_char: char,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourcesConfig {
    pub directory: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InputConfig {
    pub keybindings: HashMap<Key, InputAction>
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum IOAdapter {
    Pancurses,
    Termion,
}

impl Config {
    pub fn new(filename: &str) -> Result<Config, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;

        let config: Result<Config, serde_json::Error> = serde_json::from_str(&file_data);
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                return Err(Error::new(ErrorKind::InvalidData, format!("{}", e)));
            }
        };

        match config.log_level.as_ref() {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(config),
            _ => Err(Error::new(ErrorKind::InvalidData,
                    format!("log_level must be one of error, warn, info, debug, or trace")))
        }
    }

    pub fn get_input_action(&self, k: KeyboardEvent) -> Option<&InputAction> {
        self.input.keybindings.get(&k.key)
    }
}
