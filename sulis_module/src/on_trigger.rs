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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all="snake_case")]
pub enum OnTrigger {
    PlayerAbility(String),
    TargetFlags(Vec<String>),
    PlayerFlags(Vec<String>),
    ShowMerchant(MerchantData),
    ShowCutscene(String),
    StartConversation(String),
    FireScript(ScriptData),
    SayLine(String),
    GameOverWindow(String),
}
