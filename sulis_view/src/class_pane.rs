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
use sulis_core::widgets::{TextArea};
use sulis_module::Class;

use crate::item_button::add_bonus_text_args;

pub const NAME: &str = "class_pane";

pub struct ClassPane {
    class: Option<Rc<Class>>,
}

impl ClassPane {
    pub fn empty() -> Rc<RefCell<ClassPane>> {
        Rc::new(RefCell::new(ClassPane {
            class: None,
        }))
    }

    pub fn new(class: Rc<Class>) -> Rc<RefCell<ClassPane>> {
        Rc::new(RefCell::new(ClassPane {
            class: Some(class),
        }))
    }

    pub fn clear_class(&mut self) {
        self.class = None;
    }

    pub fn set_class(&mut self, class: Rc<Class>) {
        self.class = Some(class);
    }
}

impl WidgetKind for ClassPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let class = match self.class {
            None => return Vec::new(),
            Some(ref class) => class,
        };

        let details = Widget::with_theme(TextArea::empty(), "details");
        {
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("name", &class.name);
            state.add_text_arg("description", &class.description);

            add_bonus_text_args(&class.bonuses_per_level, state);
        }
        vec![details]
    }
}
