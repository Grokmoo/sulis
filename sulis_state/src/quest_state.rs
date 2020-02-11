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

use crate::{save_state::QuestSaveState, ChangeListenerList};
use sulis_module::{on_trigger::QuestEntryState, Module};

pub struct QuestStateSet {
    quests: HashMap<String, QuestState>,
    current_quest: Vec<String>, // the order that quests have been updated, most recent last

    pub listeners: ChangeListenerList<QuestStateSet>,
}

impl Clone for QuestStateSet {
    fn clone(&self) -> QuestStateSet {
        QuestStateSet {
            quests: self.quests.clone(),
            current_quest: self.current_quest.clone(),
            listeners: ChangeListenerList::default(),
        }
    }
}

impl Default for QuestStateSet {
    fn default() -> QuestStateSet {
        let mut quests = HashMap::new();

        for quest in Module::all_quests() {
            let id = quest.id.to_string();
            quests.insert(id, QuestState::new(quest.id.to_string()));
        }

        QuestStateSet {
            quests,
            current_quest: Vec::new(),
            listeners: ChangeListenerList::default(),
        }
    }
}

impl QuestStateSet {
    pub fn load(data: QuestSaveState) -> QuestStateSet {
        let mut quests = HashMap::new();
        for state in data.quests {
            let id = state.id.to_string();
            quests.insert(id, state);
        }
        QuestStateSet {
            quests,
            current_quest: data.current_quest,
            listeners: ChangeListenerList::default(),
        }
    }

    pub(crate) fn current_quest_stack(&self) -> Vec<String> {
        self.current_quest.clone()
    }

    pub fn current_quest(&self) -> Option<&String> {
        self.current_quest.last()
    }

    pub fn quest(&self, quest: &str) -> Option<&QuestState> {
        self.quests.get(quest)
    }

    pub fn state(&self, quest: &str) -> QuestEntryState {
        if let Some(ref quest) = self.quests.get(quest) {
            quest.state
        } else {
            QuestEntryState::Hidden
        }
    }

    pub fn entry_state(&self, quest: &str, entry: &str) -> QuestEntryState {
        if let Some(ref quest) = self.quests.get(quest) {
            quest.entry_state(entry)
        } else {
            QuestEntryState::Hidden
        }
    }

    fn set_current_quest_and_notify(&mut self, quest: &str) {
        self.current_quest.retain(|id| id != quest);

        match self.quests.get(quest).unwrap().state {
            QuestEntryState::Complete | QuestEntryState::Hidden => {
                // don't add the current quest as active since it isn't
                // displayed in the window by default
            }
            QuestEntryState::Visible | QuestEntryState::Active => {
                self.current_quest.push(quest.to_string());
            }
        };

        self.listeners.notify(&self);
    }

    pub fn set_state(&mut self, quest_id: &str, state: QuestEntryState) {
        let mut done = false;
        if let Some(ref mut quest) = self.quests.get_mut(quest_id) {
            quest.state = state;
            done = true;
        }

        if done {
            self.set_current_quest_and_notify(quest_id);
            return;
        }

        let mut quest = QuestState::new(quest_id.to_string());
        quest.state = state;
        self.quests.insert(quest_id.to_string(), quest);
        self.set_current_quest_and_notify(quest_id);
    }

    pub fn set_entry_state(&mut self, quest_id: &str, entry: &str, state: QuestEntryState) {
        let mut done = false;
        if let Some(ref mut quest) = self.quests.get_mut(quest_id) {
            quest.set_entry_state(entry, state);
            done = true;
        }

        if done {
            self.set_current_quest_and_notify(quest_id);
            return;
        }

        let mut quest = QuestState::new(quest_id.to_string());
        quest.set_entry_state(entry, state);
        self.quests.insert(quest_id.to_string(), quest);
        self.set_current_quest_and_notify(quest_id);
    }

    pub fn quests_iter(self) -> impl Iterator<Item = (String, QuestState)> {
        self.quests.into_iter()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestState {
    id: String,
    state: QuestEntryState,
    entries: Vec<(String, QuestEntryState)>,
}

impl QuestState {
    pub fn new(id: String) -> QuestState {
        QuestState {
            id,
            state: QuestEntryState::Hidden,
            entries: Vec::new(),
        }
    }

    pub fn entry_state(&self, entry: &str) -> QuestEntryState {
        for (ref id, state) in self.entries.iter() {
            if id == entry {
                return *state;
            }
        }

        QuestEntryState::Hidden
    }

    pub fn set_entry_state(&mut self, entry: &str, state: QuestEntryState) {
        match state {
            QuestEntryState::Visible | QuestEntryState::Active => {
                if self.state == QuestEntryState::Hidden {
                    self.state = QuestEntryState::Visible;
                }
            }
            _ => (),
        }

        for (ref id, ref mut entry_state) in self.entries.iter_mut() {
            if id != entry {
                continue;
            }

            *entry_state = state;
            return;
        }

        self.entries.push((entry.to_string(), state));
    }

    pub fn state(&self) -> QuestEntryState {
        self.state
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, QuestEntryState)> {
        self.entries.iter()
    }
}
