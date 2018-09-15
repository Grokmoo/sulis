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

use sulis_module::{Module};

#[derive(Debug, Clone)]
pub struct QuestStateSet {
    quests: HashMap<String, QuestState>,
}

impl QuestStateSet {
    pub fn load(data: Vec<QuestState>) -> QuestStateSet {
        let mut quests = HashMap::new();
        for state in data {
            let id = state.id.to_string();
            quests.insert(id, state);
        }
        QuestStateSet { quests }
    }

    pub fn new() -> QuestStateSet {
        let mut quests = HashMap::new();

        for quest in Module::all_quests() {
            let id = quest.id.to_string();
            quests.insert(id, QuestState::new(quest.id.to_string()));
        }

        QuestStateSet { quests }
    }

    pub fn state(&self, quest: &str) -> EntryState {
        if let Some(ref quest) = self.quests.get(quest) {
            quest.state
        } else {
            EntryState::Hidden
        }
    }

    pub fn entry_state(&self, quest: &str, entry: &str) -> EntryState {
        if let Some(ref quest) = self.quests.get(quest) {
            quest.entry_state(entry)
        } else {
            EntryState::Hidden
        }
    }

    pub fn set_state(&mut self, quest: &str, state: EntryState) {
        if let Some(ref mut quest) = self.quests.get_mut(quest) {
            quest.state = state;
            return;
        }

        let id = quest.to_string();
        let mut quest = QuestState::new(quest.to_string());
        quest.state = state;
        self.quests.insert(id, quest);
    }

    pub fn set_entry_state(&mut self, quest: &str, entry: &str, state: EntryState) {
        if let Some(ref mut quest) = self.quests.get_mut(quest) {
            quest.set_entry_state(entry, state);
            return;
        }

        let id = quest.to_string();
        let mut quest = QuestState::new(id.to_string());
        quest.set_entry_state(entry, state);
        self.quests.insert(id, quest);
    }

    pub fn into_iter(self) -> impl Iterator<Item=(String, QuestState)> {
        self.quests.into_iter()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestState {
    id: String,
    state: EntryState,
    entries: HashMap<String, EntryState>,
}

impl QuestState {
    pub fn new(id: String) -> QuestState {
        QuestState {
            id,
            state: EntryState::Hidden,
            entries: HashMap::new(),
        }
    }

    pub fn entry_state(&self, entry: &str) -> EntryState {
        if let Some(entry) = self.entries.get(entry) {
            *entry
        } else {
            EntryState::Hidden
        }
    }

    pub fn set_entry_state(&mut self, entry: &str, state: EntryState) {
        self.entries.insert(entry.to_string(), state);
    }

    pub fn state(&self) -> EntryState {
        self.state
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &EntryState)> {
        self.entries.iter()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EntryState {
    Hidden,
    Visible,
    Active,
}

impl EntryState {
    pub fn from_str(s: &str) -> EntryState {
        match s {
            "Hidden" => EntryState::Hidden,
            "Visible" => EntryState::Visible,
            "Active" => EntryState::Active,
            _ => {
                warn!("Invalid quest state '{}'", s);
                EntryState::Hidden
            }
        }
    }
}
