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

use sulis_rules::Attribute;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Label, Spinner};

use CharacterBuilder;
use character_builder::BuilderPane;

pub const NAME: &str = "attribute_selector_pane";

pub struct AttributeSelectorPane {
    available: i32,
    points_label: Rc<RefCell<Widget>>,
    attrs_spinners: Vec<Rc<RefCell<Spinner>>>,
}

impl AttributeSelectorPane {
    pub fn new() -> Rc<RefCell<AttributeSelectorPane>> {
        let points_label = Widget::with_theme(Label::empty(), "points_label");
        let attrs_spinners = Vec::new();

        Rc::new(RefCell::new(AttributeSelectorPane {
            points_label,
            attrs_spinners,
            available: 10,
        }))
    }
}

impl BuilderPane for AttributeSelectorPane {
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

impl WidgetKind for AttributeSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        let mut total = 0;
        for spinner in self.attrs_spinners.iter() {
            total += spinner.borrow().value();
        }
        self.available = 90 - total;

        self.points_label.borrow_mut().state.add_text_arg("points", &self.available.to_string());
        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children = Vec::new();

        let title = Widget::with_theme(Label::empty(), "title");
        children.push(title);

        self.attrs_spinners.clear();
        for attr in Attribute::iter() {
            let spinner = Spinner::new(10, 3, 18);
            self.attrs_spinners.push(Rc::clone(&spinner));
            let widget = Widget::with_theme(spinner, &format!("{}_spinner", attr.short_name()));
            widget.borrow_mut().state.add_callback(Callback::invalidate_parent_layout());
            children.push(widget);
            children.push(Widget::with_theme(Label::empty(), &format!("{}_label", attr.short_name())));
        }

        children.push(Rc::clone(&self.points_label));

        children
    }
}
