use std::io::Error;

use grt::resource::ResourceBuilder;
use grt::util::invalid_data_error;
use grt::serde_json;
use grt::serde_yaml;

pub struct Class {
    pub id: String,
    pub name: String,
}

impl PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        self.id == other.id
    }
}

impl Class {
    pub fn new(builder: ClassBuilder) -> Result<Class, Error> {
        Ok(Class {
            id: builder.id,
            name: builder.name,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ClassBuilder {
    pub id: String,
    pub name: String,
}

impl ResourceBuilder for ClassBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ClassBuilder, Error> {
        let resource: ClassBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ClassBuilder, Error> {
        let resource: Result<ClassBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
