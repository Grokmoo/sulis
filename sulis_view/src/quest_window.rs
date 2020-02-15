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

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind, RcRfc};
use sulis_core::widgets::{Button, Label, ScrollDirection, ScrollPane, TextArea};
use sulis_module::{on_trigger::QuestEntryState, Module, Quest};
use sulis_state::{ChangeListener, GameState};

pub const NAME: &str = "quest_window";

pub struct QuestWindow {
    active_quest: Option<Rc<Quest>>,
    show_completed: bool,
}

impl QuestWindow {
    pub fn new() -> RcRfc<QuestWindow> {
        Rc::new(RefCell::new(QuestWindow {
            active_quest: None,
            show_completed: false,
        }))
    }
}

impl WidgetKind for QuestWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        GameState::add_quest_state_change_listener(ChangeListener::invalidate(NAME, widget));

        let quests = GameState::quest_state();

        if self.active_quest.is_none() {
            if let Some(ref current_id) = quests.current_quest() {
                if let Some(quest) = Module::quest(current_id) {
                    self.active_quest = Some(quest);
                }
            }
        }

        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<QuestWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let show_completed_toggle = Widget::with_theme(Button::empty(), "show_completed_toggle");
        show_completed_toggle
            .borrow_mut()
            .state
            .set_active(self.show_completed);
        show_completed_toggle
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<QuestWindow>(widget);
                let cur = window.show_completed;
                window.show_completed = !cur;
                window.active_quest = None;
                parent.borrow_mut().invalidate_children();
            })));

        let show_completed_label = Widget::with_theme(Label::empty(), "show_completed_label");

        let quest_list_pane = ScrollPane::new(ScrollDirection::Vertical);
        let quest_list_widget = Widget::with_theme(quest_list_pane.clone(), "quest_list");

        let mut all_quests = Module::all_quests();
        all_quests.sort_unstable_by_key(|q| q.name.clone());

        for quest in all_quests {
            let selected = if let Some(ref active_quest) = self.active_quest {
                Rc::ptr_eq(active_quest, &quest)
            } else {
                false
            };

            let active = match quests.state(&quest.id) {
                QuestEntryState::Hidden => continue,
                QuestEntryState::Visible => selected,
                QuestEntryState::Active => true,
                QuestEntryState::Complete => {
                    if !self.show_completed {
                        continue;
                    }
                    selected
                }
            };

            let button = Widget::with_theme(Button::empty(), "quest_button");
            button.borrow_mut().state.set_active(active);

            let quest_ref = Rc::clone(&quest);
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (window, quest_window) = Widget::parent_mut::<QuestWindow>(widget);
                    quest_window.active_quest = Some(Rc::clone(&quest_ref));
                    window.borrow_mut().invalidate_children();
                })));

            let text_area = Widget::with_defaults(TextArea::empty());
            text_area
                .borrow_mut()
                .state
                .add_text_arg("name", &quest.name);

            if let QuestEntryState::Complete = quests.state(&quest.id) {
                text_area
                    .borrow_mut()
                    .state
                    .add_text_arg("complete", "true");
            }

            Widget::add_child_to(&button, text_area);

            quest_list_pane.borrow().add_to_content(button);
        }

        let quest_entries_pane = ScrollPane::new(ScrollDirection::Vertical);
        let quest_entries_widget = Widget::with_theme(quest_entries_pane.clone(), "quest_entries");

        if let Some(ref quest) = self.active_quest {
            if let Some(ref quest_state) = quests.quest(&quest.id) {
                for (id, _quest_entry) in quest_state.iter().rev() {
                    let active = match quests.entry_state(&quest.id, id) {
                        QuestEntryState::Hidden => continue,
                        QuestEntryState::Visible => false,
                        QuestEntryState::Active => true,
                        QuestEntryState::Complete => false,
                    };

                    let entry = Widget::with_theme(TextArea::empty(), "quest_entry");

                    {
                        let state = &mut entry.borrow_mut().state;
                        state.set_active(active);

                        if let Some(ref quest_data) = quest.entries.get(id) {
                            state.add_text_arg("description", &quest_data.description);
                        }
                    }

                    quest_entries_pane.borrow().add_to_content(entry);
                }
            }
        }

        vec![
            close,
            quest_list_widget,
            quest_entries_widget,
            show_completed_toggle,
            show_completed_label,
        ]
    }
}
