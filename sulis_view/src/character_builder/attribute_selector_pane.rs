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

use sulis_rules::{Attribute, AttributeList};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_module::Module;
use sulis_widgets::{Label, Spinner};

use CharacterBuilder;
use character_builder::BuilderPane;

pub const NAME: &str = "attribute_selector_pane";

pub struct AttributeSelectorPane {
    available: i32,
    attrs: AttributeList,
}

impl AttributeSelectorPane {
    pub fn new() -> Rc<RefCell<AttributeSelectorPane>> {
        let rules = Module::rules();
        let attrs = AttributeList::new(rules.base_attribute as u8);

        let total = rules.base_attribute * (Attribute::iter().count() as i32);
        let available = rules.builder_attribute_points - total;

        Rc::new(RefCell::new(AttributeSelectorPane {
            attrs,
            available,
        }))
    }

    fn calculate_available(&mut self) {
        let rules = Module::rules();

        let mut total = 0;
        for attr in Attribute::iter() {
            total += self.attrs.get(*attr) as i32;
        }
        self.available = rules.builder_attribute_points - total;
    }
}

impl BuilderPane for AttributeSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder) {
        builder.attributes = None;
        builder.prev.borrow_mut().state.set_enabled(true);
        self.calculate_available();
        builder.next.borrow_mut().state.set_enabled(self.available == 0);
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.attributes = Some(self.attrs);
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.prev(&widget);
    }
}

impl WidgetKind for AttributeSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let rules = Module::rules();
        let mut children = Vec::new();

        let title = Widget::with_theme(Label::empty(), "title");
        children.push(title);

        for attr in Attribute::iter() {
            let value = self.attrs.get(*attr) as i32;
            let max = if self.available > 0 {
                rules.builder_max_attribute
            } else {
                value
            };

            let spinner = Spinner::new(value, rules.builder_min_attribute, max);
            let widget = Widget::with_theme(spinner, &format!("{}_spinner", attr.short_name()));
            widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, kind| {
                let value = Widget::downcast_mut::<Spinner>(kind).value();

                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().invalidate_children();

                let pane = Widget::downcast_kind_mut::<AttributeSelectorPane>(&parent);
                pane.attrs.set(*attr, value as u8);
                pane.calculate_available();

                let builder_widget = Widget::get_parent(&parent);
                let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);
                builder.next.borrow_mut().state.set_enabled(pane.available == 0);
            })));
            children.push(widget);
            children.push(Widget::with_theme(Label::empty(), &format!("{}_label", attr.short_name())));
        }

        let points_label = Widget::with_theme(Label::empty(), "points_label");
        points_label.borrow_mut().state.add_text_arg("points", &self.available.to_string());
        children.push(points_label);

        children
    }
}
