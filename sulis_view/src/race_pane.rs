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

use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::widgets::TextArea;
use sulis_module::Race;

use crate::item_button::{add_attack_text_args, add_bonus_text_args};

pub const NAME: &str = "race_pane";

pub struct RacePane {
    race: Option<Rc<Race>>,
}

impl RacePane {
    pub fn empty() -> Rc<RefCell<RacePane>> {
        Rc::new(RefCell::new(RacePane { race: None }))
    }

    pub fn new(race: Rc<Race>) -> Rc<RefCell<RacePane>> {
        Rc::new(RefCell::new(RacePane { race: Some(race) }))
    }

    pub fn clear_race(&mut self) {
        self.race = None;
    }

    pub fn set_race(&mut self, race: Rc<Race>) {
        self.race = Some(race);
    }
}

impl WidgetKind for RacePane {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let race = match self.race {
            None => return Vec::new(),
            Some(ref race) => race,
        };

        let details = Widget::with_theme(TextArea::empty(), "details");
        {
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("name", &race.name);
            state.add_text_arg("description", &race.description);

            add_bonus_text_args(&race.base_stats, state);
            add_attack_text_args(&race.base_attack, state);
        }
        vec![details]
    }
}
