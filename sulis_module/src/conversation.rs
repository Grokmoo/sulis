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

use sulis_core::resource::ResourceBuilder;
use sulis_core::util::{invalid_data_error, unable_to_create_error};
use sulis_core::serde_yaml;

use {Module};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MerchantData {
    pub id: String,
    pub loot_list: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct OnSelect {
    pub target_flags: Option<Vec<String>>,
    pub player_flags: Option<Vec<String>>,
    pub show_merchant: Option<MerchantData>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub text: String,
    pub to: Option<String>,
    pub on_select: Option<OnSelect>,
    pub to_view: Option<OnSelect>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Node {
    text: String,
    on_view: Option<OnSelect>,
    responses: Vec<Response>,
}

pub struct Conversation {
    id: String,
    nodes: HashMap<String, Node>,
    initial_node: String,
}

impl PartialEq for Conversation {
    fn eq(&self, other: &Conversation) -> bool {
        self.id == other.id
    }
}

impl Conversation {
    pub fn new(builder: ConversationBuilder, _module: &Module) -> Result<Conversation, Error> {
        if !builder.nodes.contains_key(&builder.initial_node) {
            warn!("Invalid initial node for conversation.  Must be a node ID");
            return unable_to_create_error("conversation", &builder.id);
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
            initial_node: builder.initial_node,
        })
    }

    pub fn initial_node(&self) -> String {
        self.initial_node.clone()
    }

    pub fn initial_text(&self) -> &str {
        &self.nodes.get(&self.initial_node).unwrap().text
    }

    pub fn initial_responses(&self) -> &Vec<Response> {
        &self.nodes.get(&self.initial_node).unwrap().responses
    }

    // TODO don't panic when getting a node.

    pub fn on_view(&self, node: &str) -> &Option<OnSelect> {
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
    initial_node: String,
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
