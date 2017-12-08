// logger functionality
#[macro_use] extern crate log;
extern crate flexi_logger;

// json parser library for use in data
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate serde_derive;

// terminal display library
extern crate pancurses;
extern crate termion;

pub mod resource;

pub mod io;

pub mod state;

pub mod config;

pub mod animation;

pub mod ui;
