use std::io::{Read, Error, ErrorKind};
use std::fs::File;

use toml;

#[derive(Debug, Deserialize)]
pub enum IOAdapter {
    Pancurses,
    Termion,
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub display: DisplayConfig,
  pub resources: ResourcesConfig,
}

#[derive(Debug, Deserialize)]
pub struct DisplayConfig {
    pub adapter: IOAdapter,
}

#[derive(Debug, Deserialize)]
pub struct ResourcesConfig {
    pub directory: String,
}

impl Config {
    pub fn new(filename: &str) -> Result<Config, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;

        let config: Result<Config, toml::de::Error> = toml::from_str(&file_data);
        match config {
            Ok(config) => Ok(config),
            Err(e) => {
                Err(Error::new(ErrorKind::InvalidData, format!("{}", e)))
            }
        }
    }
}
