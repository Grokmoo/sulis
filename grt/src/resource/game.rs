use std::io::{Error, ErrorKind};

use resource::ResourceBuilder;
use util::Point;

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug)]
pub struct Game {
    pub starting_area: String,
    pub starting_location: Point,
    pub pc: String,
}

impl ResourceBuilder for Game {
    fn owned_id(&self) -> String {
        "Game".to_string()
    }

    fn from_json(data: &str) -> Result<Game, Error> {
        let game: Game = serde_json::from_str(data)?;

        Ok(game)
    }

    fn from_yaml(data: &str) -> Result<Game, Error> {
        let resource: Result<Game, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
