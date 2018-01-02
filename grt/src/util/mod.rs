mod point;
pub use self::point::Point;

use std::io::{Error, ErrorKind};

pub fn invalid_data_error<T>(str: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::InvalidData, str))
}
