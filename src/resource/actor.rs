use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::collections::HashMap;

use resource::ResourceBuilder;
use resource::Size;

use serde_json;

pub struct Actor {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub default_size: Rc<Size>,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn new(builder: ActorBuilder, sizes: &HashMap<usize, Rc<Size>>) -> Result<Actor, Error> {

        let default_size = match sizes.get(&builder.default_size) {
            None => {
                warn!("No match found for size '{}'", builder.default_size);
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Unable to create actor '{}'", builder.id)));
            },
            Some(size) => Rc::clone(size)
        };



        Ok(Actor {
            id: builder.id,
            player: builder.player,
            display: builder.display,
            default_size: default_size,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActorBuilder {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub default_size: usize,
}

impl ResourceBuilder for ActorBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<ActorBuilder, Error> {
        let actor: ActorBuilder = serde_json::from_str(data)?;
        Ok(actor)
    }
}
