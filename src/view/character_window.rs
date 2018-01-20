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

use std::rc::Rc;
use std::cell::RefCell;

use rules::Attribute;
use state::{ChangeListener, EntityState};
use grt::ui::{Button, Callback, Label, TextArea, Widget, WidgetKind};

pub const NAME: &str = "character_window";

pub struct CharacterWindow {
    character: Rc<RefCell<EntityState>>,
}

impl CharacterWindow {
    pub fn new(character: &Rc<RefCell<EntityState>>) -> Rc<CharacterWindow> {
        Rc::new(CharacterWindow {
            character: Rc::clone(character)
        })
    }
}

impl WidgetKind for CharacterWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&self) {
        self.character.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed character window.");
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.character.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.id);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let details = Widget::with_theme(TextArea::empty(), "details");
        {
            let pc = self.character.borrow();
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("name", &pc.actor.actor.name);
            state.add_text_arg("race", &pc.actor.actor.race.name);
            state.add_text_arg("sex", &pc.actor.actor.sex.to_string());

            for attribute in Attribute::iter() {
                state.add_text_arg(attribute.short_name(), &pc.actor.attributes.get(attribute).to_string())
            }

            let mut index = 0;
            for &(ref class, level) in pc.actor.actor.levels.iter() {
                state.add_text_arg(&format!("class_{}", index), &class.name);
                state.add_text_arg(&format!("level_{}", index), &level.to_string());

                index += 1;
            }

            state.add_text_arg("damage_min", &pc.actor.stats.damage.min.to_string());
            state.add_text_arg("damage_max", &pc.actor.stats.damage.max.to_string());

            state.add_text_arg("cur_hp", &pc.actor.hp().to_string());
            state.add_text_arg("max_hp", &pc.actor.stats.max_hp.to_string());
            state.add_text_arg("cur_ap", &pc.actor.ap().to_string());

            state.add_text_arg("initiative", &pc.actor.stats.initiative.to_string());
        }
        vec![title, close, details]
    }
}
