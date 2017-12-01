use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

#[derive(Debug)]
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

impl Tile {
    pub fn new(builder: TileBuilder) -> Tile {
        Tile {
            id: builder.id,
            name: builder.name,
            display: builder.display,
            passable: builder.passable
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileBuilder {
    pub id: String,
    pub name: String,
    pub display: char,
    pub passable: bool,
}

impl ResourceBuilder for TileBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<TileBuilder, Error> {
        let builder: TileBuilder = serde_json::from_str(data)?;

        Ok(builder)
    }
}
