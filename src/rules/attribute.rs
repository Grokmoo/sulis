#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Attribute {
    Strength,
    Dexterity,
    Endurance,
    Perception,
    Intellect,
    Wisdom,
}

use self::Attribute::*;

const ATTRS_LIST: [Attribute; 6] = [ Strength, Dexterity, Endurance, Perception, Intellect, Wisdom ];

pub const BASE_VALUE: u8 = 10;

impl Attribute {
    pub fn from(text: &str) -> Option<Attribute> {
        Some(match text {
            "Strength" => Strength,
            "Dexterity" => Dexterity,
            "Endurance" => Endurance,
            "Perception" => Perception,
            "Intellect" => Intellect,
            "Wisdom" => Wisdom,
            _ => return None,
        })
    }

    pub fn name(&self) -> &str {
        match *self {
            Strength => "Strength",
            Dexterity => "Dexterity",
            Endurance => "Endurance",
            Perception => "Perception",
            Intellect => "Intellect",
            Wisdom => "Wisdom",
        }
    }

    pub fn short_name(&self) -> &str {
        match *self {
            Strength => "str",
            Dexterity => "dex",
            Endurance => "end",
            Perception => "per",
            Intellect => "int",
            Wisdom => "wis",
        }
    }

    pub fn iter() -> AttributeIterator {
        AttributeIterator { index: 0 }
    }
}

pub struct AttributeIterator {
    index: usize,
}

impl Iterator for AttributeIterator {
    type Item = Attribute;
    fn next(&mut self) -> Option<Attribute> {
        if self.index == 6 {
            None
        } else {
            let next = ATTRS_LIST[self.index];
            self.index += 1;
            Some(next)
        }
    }
}
