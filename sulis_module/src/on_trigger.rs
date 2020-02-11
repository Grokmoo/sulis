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

use crate::rules::Time;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Kind {
    Ability(String),
    Item(String), // callback is based on an item ID, not a particular
    // slot - this allows creating callbacks after the
    // consumable items has been used
    Entity,
    Script(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MerchantData {
    pub id: String,
    pub loot_list: String,
    pub buy_frac: f32,
    pub sell_frac: f32,

    #[serde(default)]
    pub refresh_time: Time,
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
pub struct ScriptMenuChoice {
    pub display: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MenuData {
    pub title: String,
    pub choices: Vec<ScriptMenuChoice>,
    pub cb_func: String,
    pub cb_kind: Kind,
    pub cb_parent: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NumFlagData {
    pub flag: String,

    #[serde(default)]
    pub val: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuestEntryState {
    Hidden,
    Visible,
    Active,
    Complete,
}

impl QuestEntryState {
    pub fn unwrap_from_str(s: &str) -> QuestEntryState {
        match s {
            "Hidden" => QuestEntryState::Hidden,
            "Visible" => QuestEntryState::Visible,
            "Active" => QuestEntryState::Active,
            "Complete" => QuestEntryState::Complete,
            _ => {
                warn!("Invalid quest state '{}'", s);
                QuestEntryState::Hidden
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QuestStateData {
    pub quest: String,
    pub entry: Option<String>,
    pub state: QuestEntryState,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModuleLoadData {
    pub module: String,
    pub party: Vec<usize>,
    pub include_stash: bool,
    pub flags: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum OnTrigger {
    BlockUI(u32), // block user interface for specified number of millis
    PlayerCoins(i32),
    PartyMember(String),
    PartyItem(String),
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
    LoadModule(ModuleLoadData),
    ShowConfirm(DialogData),
    ShowMenu(MenuData),
    QuestState(QuestStateData),
    NotQuestState(QuestStateData),
    FadeOutIn,
    CheckEndTurn,
}
