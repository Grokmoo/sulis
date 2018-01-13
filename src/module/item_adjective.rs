use std::io::{Error, ErrorKind};

use grt::resource::ResourceBuilder;
use grt::serde_json;
use grt::serde_yaml;

/// An adjective is a modifier that affects the stats of
/// an item in a given way.  Items can have zero, one, or
/// many adjectives.
#[derive(Deserialize, Debug)]
pub struct ItemAdjective {
    pub id: String,
    pub name: String,
}

impl PartialEq for ItemAdjective {
    fn eq(&self, other: &ItemAdjective) -> bool {
        self.id == other.id
    }
}

impl ResourceBuilder for ItemAdjective {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn from_json(data: &str) -> Result<ItemAdjective, Error> {
        let resource: ItemAdjective = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ItemAdjective, Error> {
        let resource: Result<ItemAdjective, serde_yaml::Error>
            = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData,
                                         format!("{}", error)))
        }
    }
}
