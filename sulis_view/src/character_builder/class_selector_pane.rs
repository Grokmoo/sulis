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

}

impl ClassSelectorPane {
    pub fn new() -> Rc<RefCell<ClassSelectorPane>> {
        Rc::new(RefCell::new(ClassSelectorPane {

        }))
    }
}

impl BuilderPane for ClassSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder) {
        builder.prev.borrow_mut().state.set_enabled(true);
        builder.next.borrow_mut().state.set_enabled(false);
    }

    fn next(&mut self, _builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
        //builder.next(&widget);
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
            let entry = match active_entry {
                None => return,
                Some(entry) => entry,
            };
            class_pane_ref.borrow_mut().set_class(Rc::clone(&entry.item().class));
            class_pane_widget_ref.borrow_mut().invalidate_children();
        }));
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
