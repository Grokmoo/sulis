use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

use resource::{Class, Item, Race, ResourceBuilder, ResourceSet, Sprite};
use util::invalid_data_error;

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug)]
pub enum Sex {
    Male,
    Female,
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Actor {
    pub id: String,
    pub name: String,
    pub player: bool,
    pub race: Rc<Race>,
    pub sex: Sex,
    pub text_display: char,
    pub image_display: Rc<Sprite>,
    pub items: Vec<Rc<Item>>,
    pub levels: Vec<(Rc<Class>, u8)>,
}

impl PartialEq for Actor {
    fn eq(&self, other: &Actor) -> bool {
        self.id == other.id
    }
}

impl Actor {
    pub fn new(builder: ActorBuilder, resources: &ResourceSet) -> Result<Actor, Error> {
        let race = match resources.races.get(&builder.race) {
            None => {
                warn!("No match found for race '{}'", builder.race);
                return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
            }, Some(race) => Rc::clone(race)
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

        let sex = match builder.sex {
            None => Sex::Male,
            Some(sex) => sex,
        };

        let mut levels: Vec<(Rc<Class>, u8)> = Vec::new();
        for (class_id, level) in builder.levels {
            let class = match resources.classes.get(&class_id) {
                None => {
                    warn!("No match for class '{}'", class_id);
                    return invalid_data_error(&format!("Unable to create actor '{}'", builder.id));
                }, Some(class) => Rc::clone(class)
            };

            levels.push((class, level));
        }

        let sprite = resources.get_sprite(&builder.image_display)?;

        Ok(Actor {
            id: builder.id,
            name: builder.name,
            player: builder.player.unwrap_or(false),
            text_display: builder.text_display,
            image_display: sprite,
            race,
            sex,
            levels,
            items,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ActorBuilder {
    pub id: String,
    pub name: String,
    pub race: String,
    pub sex: Option<Sex>,
    pub player: Option<bool>,
    pub text_display: char,
    pub image_display: String,
    pub items: Option<Vec<String>>,
    pub levels: HashMap<String, u8>,
}

impl ResourceBuilder for ActorBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ActorBuilder, Error> {
        let resource: ActorBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ActorBuilder, Error> {
        let resource: Result<ActorBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
