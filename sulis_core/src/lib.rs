//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

extern crate backtrace;

#[macro_use] extern crate lazy_static;

#[macro_use] extern crate log;
extern crate flexi_logger;

extern crate serde;
pub extern crate serde_json;
pub extern crate serde_yaml;
#[macro_use] extern crate serde_derive;

#[macro_use] extern crate glium;
pub extern crate image as extern_image;

pub mod config;
pub mod image;
pub mod io;
pub mod resource;
pub mod ui;
pub mod util;
