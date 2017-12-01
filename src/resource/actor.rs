use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub default_size: usize,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl ResourceBuilder for Actor {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<Actor, Error> {
        let actor: Actor = serde_json::from_str(data)?;
        Ok(actor)
    }
}
