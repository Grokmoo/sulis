use std::io::{Read, Error, ErrorKind};
use std::path::Path;
use std::fs::{self, File};
use std::collections::HashMap;

use io::keyboard_event::Key;
use io::{KeyboardEvent, InputAction};

use serde_yaml;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
  pub input: InputConfig,
  pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub log_level: String,
    pub use_timestamps: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
    pub frame_rate: u32,
    pub animation_base_time_millis: u32,
    pub width: i32,
    pub height: i32,
    pub cursor_char: char,
    pub width_pixels: u32,
    pub height_pixels: u32,
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
    Auto,
    Pancurses,
    Termion,
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
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!("Unable to create configuration file '{}'", filename);
                    eprintln!("Exiting...");
                    ::std::process::exit(1);
                }
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
