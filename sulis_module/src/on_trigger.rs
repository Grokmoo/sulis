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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MerchantData {
    pub id: String,
    pub loot_list: String,
    pub buy_frac: f32,
    pub sell_frac: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScriptData {
    pub id: String,
    pub func: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct OnTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_ability: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_flags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_flags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_merchant: Option<MerchantData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_cutscene: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_conversation: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fire_script: Option<ScriptData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub say_line: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_over_window: Option<String>,
}
