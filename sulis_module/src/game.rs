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

use std::rc::Rc;
use std::io::Error;

use sulis_core::resource::ResourceBuilder;
use sulis_core::util::{invalid_data_error, unable_to_create_error, Point};
use sulis_core::serde_yaml;

use {Conversation, Module};

pub struct Game {
    pub starting_area: String,
    pub starting_location: Point,
    pub name: String,
    pub description: String,
    pub backstory_conversation: Rc<Conversation>,
}

impl Game {
    pub fn new(builder: GameBuilder) -> Result<Game, Error> {

        let backstory_conversation = match Module::conversation(&builder.backstory_conversation) {
            None => {
                warn!("Backstory conversation '{}' not found", &builder.backstory_conversation);
                return unable_to_create_error("module", &builder.name);
            }, Some(convo) => convo,
        };

        Ok(Game {
            starting_area: builder.starting_area,
            starting_location: builder.starting_location,
            name: builder.name,
            description: builder.description,
            backstory_conversation,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GameBuilder {
    pub starting_area: String,
    pub starting_location: Point,
    pub name: String,
    pub description: String,
    pub backstory_conversation: String,
}

impl ResourceBuilder for GameBuilder {
    fn owned_id(&self) -> String {
        "GameBuilder".to_string()
    }

    fn from_yaml(data: &str) -> Result<GameBuilder, Error> {
        let resource: Result<GameBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(e) => invalid_data_error(&format!("{}", e)),
        }
    }
}
