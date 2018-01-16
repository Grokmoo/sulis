extern crate image as extern_image;

extern crate grt;

extern crate rand;

// TODO ideally these should be re-exported from grt so
// we don't have to worry about version mismatch
// json parser library for use in data
extern crate serde;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

pub mod animation;

pub mod main_menu;

pub mod module;

pub mod rules;

pub mod state;

pub mod view;
