use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::{ResourceBuilder, ResourceSet};
use resource::Size;
use resource::Item;

use serde_json;

pub struct Actor {
    pub id: String,
    pub player: bool,
    pub display: char,
    pub default_size: Rc<Size>,
    pub items: Vec<Rc<Item>>,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn new(builder: ActorBuilder, resources: &ResourceSet) -> Result<Actor, Error> {

        let default_size = match resources.sizes.get(&builder.default_size) {
            None => {
                warn!("No match found for size '{}'", builder.default_size);
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Unable to create actor '{}'", builder.id)));
            },
            Some(size) => Rc::clone(size)
        };


        let mut items: Vec<Rc<Item>> = Vec::new();
        if let Some(builder_items) = builder.items {
            for item_id in builder_items {
                let item = match resources.items.get(&item_id) {
                    None => {
                        warn!("No match found for item ID '{}'", item_id);
                        return Err(Error::new(ErrorKind::InvalidData,
                             format!("Unable to create actor '{}'", builder.id)));
                    },
                    Some(item) => Rc::clone(item)
                };
                items.push(item);
            }
        }

        Ok(Actor {
            id: builder.id,
            player: builder.player.unwrap_or(false),
            display: builder.display,
            default_size: default_size,
            items,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ActorBuilder {
    pub id: String,
    pub player: Option<bool>,
    pub display: char,
    pub default_size: usize,
    pub items: Option<Vec<String>>,
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
