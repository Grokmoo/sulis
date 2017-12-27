use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::collections::HashMap;

use resource::Image;
use resource::ResourceBuilder;

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Slot {
    Head,
    Torso,
    Hands,
    HeldMain,
    HeldOff,
    Legs,
    Feet,
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub icon: Rc<Image>,
    pub slot: Option<Slot>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.id == other.id
    }
}

impl Item {
    pub fn new(builder: ItemBuilder,
               images: &HashMap<String, Rc<Image>>) -> Result<Item, Error> {
        let icon = match images.get(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Unable to create item '{}'", builder.id)));
            },
            Some(icon) => Rc::clone(icon)
        };

        Ok(Item {
            id: builder.id,
            icon: icon,
            name: builder.name,
            slot: builder.slot,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ItemBuilder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub slot: Option<Slot>,
}

impl ResourceBuilder for ItemBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ItemBuilder, Error> {
        let resource: ItemBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ItemBuilder, Error> {
        let resource: Result<ItemBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
