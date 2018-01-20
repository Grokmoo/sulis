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
