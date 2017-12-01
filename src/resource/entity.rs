use std::io::Error;

use resource::ResourceBuilder;

use serde_json;

pub struct Entity {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub size: usize,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Entity) -> bool {
        self.id == other.id
    }
}

impl Entity {
    pub fn new(builder: EntityBuilder) -> Entity {
        Entity {
            id: builder.id,
            player: builder.player,
            display: builder.display,
            size: builder.size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityBuilder {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub size: usize,
}

impl ResourceBuilder for EntityBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<EntityBuilder, Error> {
        let builder: EntityBuilder = serde_json::from_str(data)?;

        Ok(builder)
    }
}
