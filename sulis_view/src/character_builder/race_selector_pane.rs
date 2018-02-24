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
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{self, Display};

use sulis_core::ui::{Widget, WidgetKind};
use sulis_widgets::{Label, list_box, MutuallyExclusiveListBox};
use sulis_module::{Module, Race};

use {CharacterBuilder, RacePane};
use character_builder::BuilderPane;

pub const NAME: &str = "race_selector_pane";

pub struct RaceSelectorPane {
    list_box: Option<Rc<RefCell<MutuallyExclusiveListBox<RaceInfo>>>>,
}

impl RaceSelectorPane {
    pub fn new() -> Rc<RefCell<RaceSelectorPane>> {
        Rc::new(RefCell::new(RaceSelectorPane {
            list_box: None,
        }))
    }
}

impl BuilderPane for RaceSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder) {
        builder.race = None;
        let next = match self.list_box {
            None => false,
            Some(ref list_box) => list_box.borrow().has_active_entry(),
        };
        builder.prev.borrow_mut().state.set_enabled(false);
        builder.next.borrow_mut().state.set_enabled(next);
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        match self.list_box {
            Some(ref list_box) => {
                builder.race = match list_box.borrow().active_entry() {
                    Some(ref entry) => Some(Rc::clone(&entry.item().race)),
                    None => None,
                };
                builder.next(&widget);
            }, None => (),
        }
    }

    fn prev(&mut self, _builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) { }
}

impl WidgetKind for RaceSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries = Vec::new();
        for race in Module::all_races() {
            if !race.player { continue; }
            let race_info = RaceInfo { race };
            entries.push(list_box::Entry::new(race_info, None));
        }

        let race_pane = RacePane::empty();
        let race_pane_ref = Rc::clone(&race_pane);
        let race_pane_widget = Widget::with_defaults(race_pane);
        let race_pane_widget_ref = Rc::clone(&race_pane_widget);

        let list_box = MutuallyExclusiveListBox::with_callback(entries, Rc::new(move |active_entry| {
            let parent = Widget::go_up_tree(&race_pane_widget_ref, 2);
            let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&parent);

            match active_entry {
                None => {
                    race_pane_ref.borrow_mut().clear_race();
                    builder.next.borrow_mut().state.set_enabled(false);
                },
                Some(entry) => {
                    race_pane_ref.borrow_mut().set_race(Rc::clone(&entry.item().race));
                    builder.next.borrow_mut().state.set_enabled(true);
                },
            }

            race_pane_widget_ref.borrow_mut().invalidate_children();
        }));
        self.list_box = Some(list_box.clone());
        let races_list = Widget::with_theme(list_box, "races_list");

        vec![title, races_list, race_pane_widget]
    }
}

#[derive(Clone)]
struct RaceInfo {
    race: Rc<Race>,
}

impl Display for RaceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.race.name)
    }
}
