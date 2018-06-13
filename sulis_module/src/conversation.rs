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

use std::slice::Iter;
use std::collections::HashMap;
use std::io::Error;

use sulis_core::resource::ResourceBuilder;
use sulis_core::util::{invalid_data_error, unable_to_create_error};
use sulis_core::serde_yaml;

use {Module, OnTrigger};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub text: String,
    pub to: Option<String>,
    pub on_select: Option<OnTrigger>,
    pub to_view: Option<OnTrigger>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Node {
    text: String,
    on_view: Option<OnTrigger>,
    responses: Vec<Response>,
}

#[derive(Debug)]
pub struct Conversation {
    id: String,
    nodes: HashMap<String, Node>,
    initial_nodes: Vec<(String, Option<OnTrigger>)>,
}

impl PartialEq for Conversation {
    fn eq(&self, other: &Conversation) -> bool {
        self.id == other.id
    }
}

impl Conversation {
    pub fn new(builder: ConversationBuilder, _module: &Module) -> Result<Conversation, Error> {
        if builder.initial_nodes.is_empty() {
            warn!("Must specify at least one initial node for conversation");
            return unable_to_create_error("conversation", &builder.id);
        }

        for (node, _) in builder.initial_nodes.iter() {
            if !builder.nodes.contains_key(node) {
                warn!("Invalid initial for node '{}'", node);
                return unable_to_create_error("conversation", &builder.id);
            }
        }

        for (_, ref node) in builder.nodes.iter() {
            for response in node.responses.iter() {
                if let Some(ref to) = response.to {
                    if !builder.nodes.contains_key(to) {
                        warn!("Invalid to for node response.  Must be a node ID");
                        return unable_to_create_error("conversation", &builder.id);
                    }
                }
            }
        }

        Ok(Conversation {
            id: builder.id,
            nodes: builder.nodes,
            initial_nodes: builder.initial_nodes,
        })
    }

    pub fn initial_nodes(&self) -> Iter<(String, Option<OnTrigger>)> {
        self.initial_nodes.iter()
    }

    // TODO don't panic when getting a node.

    pub fn on_view(&self, node: &str) -> &Option<OnTrigger> {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(ref node) => &node.on_view,
        }
    }

    pub fn text(&self, node: &str) -> &str {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(ref node) => &node.text
        }
    }

    pub fn responses(&self, node: &str) -> &Vec<Response> {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(ref node) => &node.responses,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConversationBuilder {
    pub id: String,
    nodes: HashMap<String, Node>,
    initial_nodes: Vec<(String, Option<OnTrigger>)>,
}

impl ResourceBuilder for ConversationBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_yaml(data: &str) -> Result<ConversationBuilder, Error> {
        let resource: Result<ConversationBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
