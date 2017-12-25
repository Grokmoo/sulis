use std::io::Error;

#[derive(Debug, PartialEq)]
pub enum BuilderType {
    JSON,
    YAML,
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn from_json(data: &str) -> Result<Self, Error>;

    fn from_yaml(data: &str) -> Result<Self, Error>;
}
