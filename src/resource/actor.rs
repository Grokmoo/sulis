use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

pub struct Actor {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub size: usize,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn new(builder: ActorBuilder) -> Actor {
        Actor {
            id: builder.id,
            player: builder.player,
            display: builder.display,
            size: builder.size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActorBuilder {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub size: usize,
}

impl ResourceBuilder for ActorBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<ActorBuilder, Error> {
        let builder: ActorBuilder = serde_json::from_str(data)?;

        Ok(builder)
    }
}
