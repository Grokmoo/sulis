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
#[serde(deny_unknown_fields)]
pub struct DialogData {
    pub message: String,
    pub accept_text: String,
    pub cancel_text: String,
    pub on_accept: Option<ScriptData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NumFlagData {
    pub flag: String,

    #[serde(default)]
    pub val: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all="snake_case")]
pub enum OnTrigger {
    PlayerCoins(i32),
    PartyMember(String),
    PlayerNumFlag(NumFlagData),
    TargetNumFlag(NumFlagData),
    NotPlayerNumFlag(NumFlagData),
    NotTargetNumFlag(NumFlagData),
    PlayerAbility(String),
    NotPlayerFlag(String),
    NotTargetFlag(String),
    TargetFlag(String),
    PlayerFlag(String),
    ShowMerchant(MerchantData),
    ShowCutscene(String),
    StartConversation(String),
    FireScript(ScriptData),
    SayLine(String),
    GameOverWindow(String),
    ScrollView(i32, i32),
    LoadModule(String),
    ShowConfirm(DialogData),
    FadeOutIn,
}
