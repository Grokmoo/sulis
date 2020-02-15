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

use sulis_core::ui::{Widget, WidgetKind, RcRfc};
use sulis_core::widgets::{Label, TextArea};
use sulis_module::{Ability, Class};

use crate::character_builder::{attribute_selector_pane::AbilityButton, BuilderPane};
use crate::CharacterBuilder;

pub const NAME: &str = "level_up_finish_pane";

pub struct LevelUpFinishPane {
    class: Option<Rc<Class>>,
    abilities: Vec<Rc<Ability>>,
    level: u32,
}

impl LevelUpFinishPane {
    pub fn new(level: u32) -> RcRfc<LevelUpFinishPane> {
        Rc::new(RefCell::new(LevelUpFinishPane {
            class: None,
            abilities: Vec::new(),
            level,
        }))
    }
}

impl BuilderPane for LevelUpFinishPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: RcRfc<Widget>) {
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

    fn next(&mut self, _builder: &mut CharacterBuilder, _widget: RcRfc<Widget>) {}

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: RcRfc<Widget>) {
        builder.next.borrow_mut().state.set_visible(true);
        builder.finish.borrow_mut().state.set_visible(false);
        builder.prev(&widget);
    }
}

impl WidgetKind for LevelUpFinishPane {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let class = match &self.class {
            Some(class) => class,
            None => return Vec::new(),
        };

        let details = Widget::with_theme(TextArea::empty(), "details");

        {
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("class", &class.name);
            state.add_text_arg("level", &format!("{}", self.level));
        }

        let abilities = Widget::empty("abilities");
        for ability in &self.abilities {
            let class = Rc::clone(class);
            let ability = Rc::clone(ability);

            let icon = Widget::with_theme(Label::empty(), "icon");
            icon.borrow_mut()
                .state
                .add_text_arg("icon", &ability.icon.id());

            let button = Widget::with_defaults(AbilityButton::new(ability, class));

            Widget::add_child_to(&button, icon);
            Widget::add_child_to(&abilities, button);
        }

        vec![title, details, abilities]
    }
}
