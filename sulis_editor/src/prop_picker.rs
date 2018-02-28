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
use sulis_module::{Prop, Module};
use sulis_widgets::Button;

const NAME: &str = "prop_picker";

pub struct PropPicker {
    cur_prop: Option<Rc<Prop>>,
}

impl PropPicker {
    pub fn new() -> Rc<RefCell<PropPicker>> {
        Rc::new(RefCell::new(PropPicker {
            cur_prop: None,
        }))
    }

    pub fn get_cur_prop(&self) -> Option<Rc<Prop>> {
        match self.cur_prop {
            None => None,
            Some(ref prop) => Some(Rc::clone(prop)),
        }
    }
}

impl WidgetKind for PropPicker {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut all_props = Module::all_props();
        all_props.sort_by(|a, b| a.id.cmp(&b.id));

        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();
        for prop in all_props {
            let button = Widget::with_theme(Button::empty(), "prop_button");
            button.borrow_mut().state.add_text_arg("name", &prop.name);
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set active prop: {}", widget.borrow().state.text);
                    for child in parent.borrow().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);
                }

                let prop_picker = Widget::downcast_kind_mut::<PropPicker>(&parent);
                prop_picker.cur_prop = Some(Rc::clone(&prop));
            })));

            widgets.push(button);
        }

        widgets
    }
}
