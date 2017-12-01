use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub starting_area: String,
    pub starting_location: Vec<usize>,
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
