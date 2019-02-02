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

use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::widgets::{Label, TextArea};
use sulis_module::{Ability, Class};

use crate::{CharacterBuilder};
use crate::character_builder::BuilderPane;

pub const NAME: &str = "level_up_finish_pane";

pub struct LevelUpFinishPane {
    class: Option<Rc<Class>>,
    abilities: Vec<Rc<Ability>>,
}

impl LevelUpFinishPane {
    pub fn new() -> Rc<RefCell<LevelUpFinishPane>> {
        Rc::new(RefCell::new(LevelUpFinishPane {
            class: None,
            abilities: Vec::new(),
        }))
    }
}

impl BuilderPane for LevelUpFinishPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.prev.borrow_mut().state.set_enabled(true);
        builder.next.borrow_mut().state.set_enabled(false);
        builder.next.borrow_mut().state.set_visible(false);
        builder.finish.borrow_mut().state.set_visible(true);
        builder.finish.borrow_mut().state.set_enabled(true);

        self.class = match builder.class {
            None => None,
            Some(ref class) => Some(Rc::clone(class)),
        };

        self.abilities = builder.abilities.clone();

        widget.borrow_mut().invalidate_children();
    }

    fn next(&mut self, _builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.next.borrow_mut().state.set_visible(true);
        builder.finish.borrow_mut().state.set_visible(false);
        builder.prev(&widget);
    }
}

impl WidgetKind for LevelUpFinishPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let details = Widget::with_theme(TextArea::empty(), "details");

        {
            let state = &mut details.borrow_mut().state;
            if let Some(ref class) = self.class {
                state.add_text_arg("class", &class.name);
            }

            for (index, ability) in self.abilities.iter().enumerate() {
                state.add_text_arg(&format!("ability_name_{}", index), &ability.name);
            }
        }

        vec![title, details]
    }
}
