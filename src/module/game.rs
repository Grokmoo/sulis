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

use std::io::Error;

use grt::resource::ResourceBuilder;
use grt::util::{invalid_data_error, Point};
use grt::serde_json;
use grt::serde_yaml;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub starting_area: String,
    pub starting_location: Point,
    pub pc: String,
    pub name: String,
}

impl ResourceBuilder for Game {
    fn owned_id(&self) -> String {
        "Game".to_string()
    }

    fn from_json(data: &str) -> Result<Game, Error> {
        let game: Game = serde_json::from_str(data)?;

        Ok(game)
    }

    fn from_yaml(data: &str) -> Result<Game, Error> {
        let resource: Result<Game, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(e) => invalid_data_error(&format!("{}", e)),
        }
    }
}
