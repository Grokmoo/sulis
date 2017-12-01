use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub id: String,
    pub name: String,
    pub display: char,
    pub passable: bool,
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.id == other.id
    }
}

impl ResourceBuilder for Tile {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<Tile, Error> {
        let tile: Tile = serde_json::from_str(data)?;

        Ok(tile)
    }
}
