use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::collections::HashMap;

use resource::Image;
use resource::ResourceBuilder;

use serde_json;

pub struct Item {
    pub id: String,
    pub icon: Rc<Image>,
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
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ItemBuilder {
    pub id: String,
    pub icon: String,
}

impl ResourceBuilder for ItemBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<ItemBuilder, Error> {
        let item: ItemBuilder = serde_json::from_str(data)?;

        Ok(item)
    }
}
