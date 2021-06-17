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

use std::collections::HashMap;
use std::io::Error;
use std::slice::Iter;

use sulis_core::util::unable_to_create_error;

use crate::{Module, OnTrigger};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub text: String,
    pub to: Option<String>,

    #[serde(default)]
    pub on_select: Vec<OnTrigger>,

    #[serde(default)]
    pub to_view: Vec<OnTrigger>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Node {
    text: String,

    #[serde(default)]
    switch_speaker: Option<String>,

    #[serde(default)]
    on_view: Vec<OnTrigger>,
    responses: Vec<Response>,
}

#[derive(Debug)]
pub struct Conversation {
    pub id: String,
    nodes: HashMap<String, Node>,
    initial_nodes: Vec<(String, Vec<OnTrigger>)>,
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

        let mut initial_nodes = Vec::new();
        for node in builder.initial_nodes {
            if !builder.nodes.contains_key(&node.id) {
                warn!("Invalid initial node '{}'", node.id);
                return unable_to_create_error("conversation", &builder.id);
            }

            initial_nodes.push((node.id, node.to_view));
        }

        for (_, node) in builder.nodes.iter() {
            for response in node.responses.iter() {
                if let Some(ref to) = response.to {
                    if !builder.nodes.contains_key(to) {
                        warn!("Invalid to '{}' for node response.  Must be a node ID", to);
                        return unable_to_create_error("conversation", &builder.id);
                    }
                }
            }
        }

        Ok(Conversation {
            id: builder.id,
            nodes: builder.nodes,
            initial_nodes,
        })
    }

    pub fn initial_nodes(&self) -> Iter<(String, Vec<OnTrigger>)> {
        self.initial_nodes.iter()
    }

    // TODO don't panic when getting a node.

    pub fn on_view(&self, node: &str) -> &Vec<OnTrigger> {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(node) => &node.on_view,
        }
    }

    pub fn switch_speaker(&self, node: &str) -> &Option<String> {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(node) => &node.switch_speaker,
        }
    }

    pub fn text(&self, node: &str) -> &str {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(node) => &node.text,
        }
    }

    pub fn responses(&self, node: &str) -> &Vec<Response> {
        match self.nodes.get(node) {
            None => panic!("Invalid node"),
            Some(node) => &node.responses,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InitialNode {
    id: String,

    #[serde(default)]
    to_view: Vec<OnTrigger>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConversationBuilder {
    pub id: String,
    nodes: HashMap<String, Node>,
    initial_nodes: Vec<InitialNode>,
}
