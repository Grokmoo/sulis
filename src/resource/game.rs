use std::io::Error;

use resource::ResourceBuilder;
use resource::Point;

use serde_json;

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

    fn new(data: &str) -> Result<Game, Error> {
        let game: Game = serde_json::from_str(data)?;

        Ok(game)
    }
}
