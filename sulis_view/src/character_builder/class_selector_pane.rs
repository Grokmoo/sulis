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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label};
use sulis_module::{Class, Module};

use crate::character_builder::BuilderPane;
use crate::{CharacterBuilder, ClassPane};

pub const NAME: &str = "class_selector_pane";

pub struct ClassSelectorPane {
    class_choices: Vec<String>,
    allow_prev: bool,
    selected_class: Option<Rc<Class>>,
}

impl ClassSelectorPane {
    pub fn new(choices: Vec<String>, allow_prev: bool) -> Rc<RefCell<ClassSelectorPane>> {
        Rc::new(RefCell::new(ClassSelectorPane {
            selected_class: None,
            class_choices: choices,
            allow_prev,
        }))
    }
}

impl BuilderPane for ClassSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.class = None;
        builder.prev.borrow_mut().state.set_enabled(self.allow_prev);
        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.selected_class.is_some());
        widget.borrow_mut().invalidate_children();
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let class = match self.selected_class {
            None => return,
            Some(ref class) => class,
        };
        builder.class = Some(Rc::clone(class));
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        if self.allow_prev {
            self.selected_class = None;
            builder.prev(&widget);
        }
    }
}

impl WidgetKind for ClassSelectorPane {
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
        let title = Widget::with_theme(Label::empty(), "title");

        let classes_pane = Widget::empty("classes_pane");
        for class_id in self.class_choices.iter() {
            let class = match Module::class(class_id) {
                None => {
                    warn!("Selectable class '{}' not found", class_id);
                    continue;
                }
                Some(class) => class,
            };

            let class_button = Widget::with_theme(Button::empty(), "class_button");
            class_button
                .borrow_mut()
                .state
                .add_text_arg("name", &class.name);
            if let Some(ref selected_class) = self.selected_class {
                class_button
                    .borrow_mut()
                    .state
                    .set_active(class == *selected_class);
            }

            let class_ref = Rc::clone(&class);
            class_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, pane) = Widget::parent_mut::<ClassSelectorPane>(widget);
                    pane.selected_class = Some(Rc::clone(&class_ref));
                    parent.borrow_mut().invalidate_children();

                    let (_, builder) = Widget::parent_mut::<CharacterBuilder>(&parent);
                    builder.next.borrow_mut().state.set_enabled(true);
                })));

            Widget::add_child_to(&classes_pane, class_button);
        }

        let class = match self.selected_class {
            None => return vec![title, classes_pane],
            Some(ref class) => class,
        };

        let class_pane = ClassPane::empty();
        class_pane.borrow_mut().set_class(Rc::clone(class));
        let class_pane_widget = Widget::with_defaults(class_pane);

        vec![title, class_pane_widget, classes_pane]
    }
}
