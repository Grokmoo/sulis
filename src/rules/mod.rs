pub mod attribute;
pub use self::attribute::Attribute;

pub mod damage;
pub use self::damage::Damage;

use self::attribute::Attribute::*;

#[derive(Clone)]
pub struct AttributeList([(Attribute, u8); 6]);

impl Default for AttributeList {
    fn default() -> AttributeList {
        AttributeList([
            (Strength, attribute::BASE_VALUE),
            (Dexterity, attribute::BASE_VALUE),
            (Endurance, attribute::BASE_VALUE),
            (Perception, attribute::BASE_VALUE),
            (Intellect, attribute::BASE_VALUE),
            (Wisdom, attribute::BASE_VALUE),
        ])
    }
}

impl AttributeList {
    pub fn get(&self, attr: Attribute) -> u8 {
        match self.0.iter().find(|a| a.0 == attr) {
            Some(val) => val.1,
            None => 0,
        }
    }

    pub fn set(&mut self, attr: Attribute, value: u8) {
        if let Some(attr) = self.0.iter_mut().find(|a| a.0 == attr) {
            attr.1 = value;
        }
    }
}

pub struct StatList {
    pub damage: Damage,
}


impl Default for StatList {
    fn default() -> StatList {
        StatList {
            damage: Damage::default(),
        }
    }
}
