use std::io::Error;
use std::rc::Rc;

use resource::{ResourceBuilder, ResourceSet, EntitySize};
use util::invalid_data_error;

use serde_json;
use serde_yaml;

pub struct Race {
    pub id: String,
    pub name: String,
    pub size: Rc<EntitySize>,
}

impl PartialEq for Race {
    fn eq(&self, other: &Race) -> bool {
        self.id == other.id
    }
}

impl Race {
    pub fn new(builder: RaceBuilder, resources: &ResourceSet) -> Result<Race, Error> {
        let size = match resources.sizes.get(&builder.size) {
            None => {
                warn!("No match found for size '{}'", builder.size);
                return invalid_data_error(&format!("Unable to create race '{}'", builder.id));
            }, Some(size) => Rc::clone(size)
        };

        Ok(Race {
            id: builder.id,
            name: builder.name,
            size,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct RaceBuilder {
    pub id: String,
    pub name: String,
    pub size: usize,
}

impl ResourceBuilder for RaceBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<RaceBuilder, Error> {
        let resource: RaceBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<RaceBuilder, Error> {
        let resource: Result<RaceBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
