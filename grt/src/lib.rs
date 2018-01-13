#[macro_use] extern crate lazy_static;

// logger functionality
#[macro_use] extern crate log;

// json parser library for use in data
extern crate serde;
pub extern crate serde_json;
pub extern crate serde_yaml;
#[macro_use] extern crate serde_derive;

// terminal display library
#[cfg(windows)] extern crate pancurses;
#[cfg(not(windows))] extern crate termion;

#[macro_use] extern crate glium;
pub extern crate image as extern_image;

pub mod config;
pub mod image;
pub mod io;
pub mod resource;
pub mod ui;
pub mod util;
