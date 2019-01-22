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

use sulis_core::util::{unable_to_create_error};

use crate::{Module};

#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub enum FuncKind {
    OnDamaged,
    BeforeAttack,
    AfterAttack,
    BeforeDefense,
}

#[derive(Debug)]
pub struct AITemplate {
    pub id: String,
    pub script: String,
    pub hooks: HashMap<FuncKind, String>,
}

impl AITemplate {
    pub fn new(builder: AITemplateBuilder, module: &Module) -> Result<AITemplate, Error> {
        match module.scripts.get(&builder.script) {
            None => {
                warn!("No script found with id '{}'", builder.script);
                return unable_to_create_error("ai_template", &builder.id);
            }, Some(_) => (),
        };

        Ok(AITemplate {
            id: builder.id,
            script: builder.script,
            hooks: builder.hooks,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AITemplateBuilder {
    pub id: String,
    pub script: String,
    pub hooks: HashMap<FuncKind, String>,
}
