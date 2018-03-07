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
use sulis_widgets::{Label, TextArea};
use sulis_module::{Class};

use {CharacterBuilder};
use character_builder::BuilderPane;

pub const NAME: &str = "level_up_finish_pane";

pub struct LevelUpFinishPane {
    class: Option<Rc<Class>>,
}

impl LevelUpFinishPane {
    pub fn new() -> Rc<RefCell<LevelUpFinishPane>> {
        Rc::new(RefCell::new(LevelUpFinishPane {
            class: None,
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

        if let Some(ref class) = self.class {
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("class", &class.name);
        }

        vec![title, details]
    }
}
