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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use sulis_module::{Module, Race};

use crate::{CharacterBuilder, RacePane};
use crate::character_builder::BuilderPane;

pub const NAME: &str = "race_selector_pane";

pub struct RaceSelectorPane {
    selected_race: Option<Rc<Race>>,
}

impl RaceSelectorPane {
    pub fn new() -> Rc<RefCell<RaceSelectorPane>> {
        Rc::new(RefCell::new(RaceSelectorPane {
            selected_race: None,
        }))
    }
}

impl BuilderPane for RaceSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
        builder.race = None;
        builder.prev.borrow_mut().state.set_enabled(false);
        builder.next.borrow_mut().state.set_enabled(self.selected_race.is_some());
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let race = match self.selected_race {
            None => return,
            Some(ref race) => race,
        };

        builder.race = Some(Rc::clone(race));
        builder.next(&widget);
    }

    fn prev(&mut self, _builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) { }
}

impl WidgetKind for RaceSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let races_pane = Widget::empty("races_pane");
        for race_id in Module::rules().selectable_races.iter() {
            let race = match Module::race(race_id) {
                None => {
                    warn!("Selectable race '{}' not found", race_id);
                    continue;
                }, Some(race) => race,
            };

            let race_button = Widget::with_theme(Button::empty(), "race_button");
            race_button.borrow_mut().state.add_text_arg("name", &race.name);
            if let Some(ref selected_race) = self.selected_race {
                race_button.borrow_mut().state.set_active(race == *selected_race);
            }

            let race_ref = Rc::clone(&race);
            race_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(&widget, 2);
                let pane = Widget::downcast_kind_mut::<RaceSelectorPane>(&parent);
                pane.selected_race = Some(Rc::clone(&race_ref));
                parent.borrow_mut().invalidate_children();

                let builder_widget = Widget::get_parent(&parent);
                let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);
                builder.next.borrow_mut().state.set_enabled(true);
            })));

            Widget::add_child_to(&races_pane, race_button);
        }

        let race = match self.selected_race {
            None => return vec![title, races_pane],
            Some(ref race) => race,
        };

        let race_pane = RacePane::empty();
        race_pane.borrow_mut().set_race(Rc::clone(race));
        let race_pane_widget = Widget::with_defaults(race_pane);

        vec![title, race_pane_widget, races_pane]
    }
}
