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

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub enum FuncKind {
    OnDamaged,
    BeforeAttack,
    AfterAttack,
    BeforeDefense,
    OnRoundElapsed,
    AiAction,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AITemplate {
    pub id: String,
    pub script: String,

    #[serde(default)]
    pub hooks: HashMap<FuncKind, String>,

    #[serde(default)]
    pub params: HashMap<String, i32>,
}
