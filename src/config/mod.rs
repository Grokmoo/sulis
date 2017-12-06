use std::io::{Read, Error, ErrorKind};
use std::fs::File;
use std::collections::HashMap;

use io::KeyboardInput;
use io::InputAction;

use serde_json;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
  pub input: InputConfig,
}

#[derive(Debug, Deserialize)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
    pub frame_rate: u32,
}

#[derive(Debug, Deserialize)]
pub struct ResourcesConfig {
    pub directory: String,
}

#[derive(Debug, Deserialize)]
pub struct InputConfig {
    pub keybindings: HashMap<KeyboardInput, InputAction>
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum IOAdapter {
    Pancurses,
}

impl Config {
    pub fn new(filename: &str) -> Result<Config, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;

        let config: Result<Config, serde_json::Error> = serde_json::from_str(&file_data);
        match config {
            Ok(config) => Ok(config),
            Err(e) => {
                Err(Error::new(ErrorKind::InvalidData, format!("{}", e)))
            }
        }
    }

    pub fn get_input_action(&self, k: KeyboardInput) -> Option<&InputAction> {
        self.input.keybindings.get(&k)
    }
}
