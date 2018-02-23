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
use sulis_module::{Module, Class};

use {CharacterBuilder, ClassPane};
use character_builder::BuilderPane;

pub const NAME: &str = "class_selector_pane";

pub struct ClassSelectorPane {
    list_box: Option<Rc<RefCell<MutuallyExclusiveListBox<ClassInfo>>>>,
}

impl ClassSelectorPane {
    pub fn new() -> Rc<RefCell<ClassSelectorPane>> {
        Rc::new(RefCell::new(ClassSelectorPane {
            list_box: None,
        }))
    }
}

impl BuilderPane for ClassSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder) {
        builder.class = None;
        let next = match self.list_box {
            None => false,
            Some(ref list_box) => list_box.borrow().has_active_entry(),
        };

        builder.prev.borrow_mut().state.set_enabled(true);
        builder.next.borrow_mut().state.set_enabled(next);
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        match self.list_box {
            None => (),
            Some(ref list_box) => {
                builder.class = match list_box.borrow().active_entry() {
                    Some(ref entry) => Some(Rc::clone(&entry.item().class)),
                    None => None,
                };
                builder.next(&widget);
            },
        }
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.prev(&widget);
    }
}

impl WidgetKind for ClassSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries = Vec::new();
        for class in Module::all_classes() {
            let class_info = ClassInfo { class };
            entries.push(list_box::Entry::new(class_info, None));
        }

        let class_pane = ClassPane::empty();
        let class_pane_ref = Rc::clone(&class_pane);
        let class_pane_widget = Widget::with_defaults(class_pane);
        let class_pane_widget_ref = Rc::clone(&class_pane_widget);

        let list_box = MutuallyExclusiveListBox::with_callback(entries, Rc::new(move |active_entry| {
            let parent = Widget::go_up_tree(&class_pane_widget_ref, 2);
            let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&parent);

            match active_entry {
                None => {
                    class_pane_ref.borrow_mut().clear_class();
                    builder.next.borrow_mut().state.set_enabled(false);
                },
                Some(entry) => {
                    class_pane_ref.borrow_mut().set_class(Rc::clone(&entry.item().class));
                    builder.next.borrow_mut().state.set_enabled(true);
                },
            };
            class_pane_widget_ref.borrow_mut().invalidate_children();
        }));
        self.list_box = Some(list_box.clone());
        let classes_list = Widget::with_theme(list_box, "classes_list");

        vec![title, classes_list, class_pane_widget]
    }
}

#[derive(Clone)]
struct ClassInfo {
    class: Rc<Class>,
}

impl Display for ClassInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.class.name)
    }
}
