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

use sulis_core::ui::{Callback, Widget, WidgetKind, RcRfc};
use sulis_core::widgets::{list_box, Label, ListBox};

pub const NAME: &str = "item_action_menu";

pub struct ItemActionMenu {
    actions: Vec<(String, Callback)>,
}

impl ItemActionMenu {
    pub fn new() -> RcRfc<ItemActionMenu> {
        Rc::new(RefCell::new(ItemActionMenu {
            actions: Vec::new(),
        }))
    }

    pub fn add_action(&mut self, name: &str, cb: Callback) {
        self.actions.push((name.to_string(), cb));
    }
}

impl WidgetKind for ItemActionMenu {
    widget_kind!(NAME);

    fn on_add(&mut self, _widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for &(ref name, ref cb) in self.actions.iter() {
            let cb = cb.clone();
            let callback = Callback::new(Rc::new(move |widget, kind| {
                cb.call(widget, kind);
                let (parent, _) = Widget::parent::<ItemActionMenu>(widget);
                parent.borrow_mut().mark_for_removal();
            }));
            let entry = list_box::Entry::new(name.to_string(), Some(callback));
            entries.push(entry);
        }
        let actions = Widget::with_theme(ListBox::new(entries), "actions");

        vec![title, actions]
    }
}
