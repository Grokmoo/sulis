// logger functionality
#[macro_use] extern crate log;
extern crate flexi_logger;

// json parser library for use in data
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

#[macro_use] extern crate serde_derive;

// terminal display library
#[cfg(windows)] extern crate pancurses;
#[cfg(not(windows))] extern crate termion;

extern crate uuid;

pub mod resource;

pub mod io;

pub mod state;

pub mod config;

pub mod animation;

pub mod ui;

pub mod view;
