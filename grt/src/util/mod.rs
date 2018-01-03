mod point;
pub use self::point::Point;

mod size;
pub use self::size::Size;

use std::io::{Error, ErrorKind};

pub fn invalid_data_error<T>(str: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::InvalidData, str))
}
