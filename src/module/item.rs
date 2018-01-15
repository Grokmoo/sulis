use std::io::{Error, ErrorKind};
use std::rc::Rc;

use grt::image::Image;
use grt::resource::{ResourceBuilder, ResourceSet};
use grt::serde_json;
use grt::serde_yaml;

use module::Equippable;

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Slot {
    Head,
    Torso,
    Hands,
    HeldMain,
    HeldOff,
    Legs,
    Feet,
}

use self::Slot::*;
const SLOTS_LIST: [Slot; 7] = [Head, Torso, Hands, HeldMain, HeldOff, Legs, Feet];

pub struct SlotIterator {
    index: usize,
}

impl Default for SlotIterator {
    fn default() -> SlotIterator {
        SlotIterator { index: 0 }
    }
}

impl Iterator for SlotIterator {
    type Item = Slot;
    fn next(&mut self) -> Option<Slot> {
        if self.index == SLOTS_LIST.len() {
            None
        } else {
            let next = SLOTS_LIST[self.index];
            self.index += 1;
            Some(next)
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub icon: Rc<Image>,
    pub equippable: Option<Equippable>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.id == other.id
    }
}

impl Item {
    pub fn new(builder: ItemBuilder) -> Result<Item, Error> {
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Unable to create item '{}'", builder.id)));
            },
            Some(icon) => icon
        };

        Ok(Item {
            id: builder.id,
            icon: icon,
            name: builder.name,
            equippable: builder.equippable,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemBuilder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub equippable: Option<Equippable>,
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
