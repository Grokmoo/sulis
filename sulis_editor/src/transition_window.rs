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

use std::str::FromStr;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_widgets::{Button, InputField, Label, list_box, ListBox};

use AreaEditor;

pub const NAME: &str = "transition_window";

fn parse_coords(input: &str) -> Option<(i32, i32)> {
    let mut vals: Vec<i32> = Vec::new();

    for s in input.split(",") {
        let i = match i32::from_str(s) {
            Err(_) => {
                warn!("Unable to parse coordinates from {}", input);
                return None;
            },
            Ok(i) => i
        };

        vals.push(i);
    }

    if vals.len() != 2 { return None; }

    Some((vals[0], vals[1]))
}

pub struct TransitionWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
    selected_transition: Option<usize>,
}

impl TransitionWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<TransitionWindow>> {
        Rc::new(RefCell::new(TransitionWindow {
            area_editor,
            selected_transition: None,
        }))
    }
}

impl WidgetKind for TransitionWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();

        let title = Widget::with_theme(Label::empty(), "title");
        widgets.push(title);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());
        widgets.push(close);

        if let Some(index) = self.selected_transition {
            let area_editor = self.area_editor.borrow();
            let transition = area_editor.transition(index);

            let from_str = format!("{},{}", transition.from.x, transition.from.y);
            let to_str = format!("{},{}", transition.to.x, transition.to.y);
            let to_area_str = match transition.to_area {
                None => "",
                Some(ref area) => area,
            };

            let from = Widget::with_theme(InputField::new(&from_str), "from");
            let to = Widget::with_theme(InputField::new(&to_str), "to");
            let to_area = Widget::with_theme(InputField::new(to_area_str), "to_area");
            let from_label = Widget::with_theme(Label::empty(), "from_label");
            let to_label = Widget::with_theme(Label::empty(), "to_label");
            let to_area_label = Widget::with_theme(Label::empty(), "to_area_label");

            let apply = Widget::with_theme(Button::empty(), "apply_button");

            let from_ref = Rc::clone(&from);
            let to_ref = Rc::clone(&to);
            let to_area_ref = Rc::clone(&to_area);
            let area_editor_ref = Rc::clone(&self.area_editor);
            apply.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let to_area_str = to_area_ref.borrow().state.text.to_string();
                let to_area = if to_area_str.is_empty() {
                    None
                } else {
                    Some(to_area_str)
                };

                let from = Point::from_tuple_i32(match parse_coords(&from_ref.borrow().state.text) {
                    None => return,
                    Some((x, y)) => (x, y),
                });

                let to = Point::from_tuple_i32(match parse_coords(&to_ref.borrow().state.text) {
                    None => return,
                    Some((x, y)) => (x, y),
                });

                let window = Widget::get_parent(widget);
                window.borrow_mut().invalidate_children();

                let transition_window = Widget::downcast_kind_mut::<TransitionWindow>(&window);
                let cur_index = match transition_window.selected_transition {
                    Some(index) => index,
                    None => return,
                };

                let mut area_editor = area_editor_ref.borrow_mut();
                let transition = area_editor.transition_mut(cur_index);
                transition.from = from;
                transition.to = to;
                transition.to_area = to_area;
            })));

            let delete = Widget::with_theme(Button::empty(), "delete_button");

            let area_editor_ref = Rc::clone(&self.area_editor);
            delete.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let window = Widget::get_parent(widget);
                window.borrow_mut().invalidate_children();

                let transition_window = Widget::downcast_kind_mut::<TransitionWindow>(&window);
                let cur_index = match transition_window.selected_transition {
                    Some(index) => index,
                    None => return,
                };

                area_editor_ref.borrow_mut().delete_transition(cur_index);
                transition_window.selected_transition = None;
            })));

            widgets.append(&mut vec![from, to, to_area, from_label, to_label,
                           to_area_label, apply, delete]);
        }

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, ref transition) in self.area_editor.borrow().transitions_iter().enumerate() {
            let cb = Callback::new(Rc::new(move |widget, _| {
                let window = Widget::go_up_tree(widget, 2);
                window.borrow_mut().invalidate_children();

                let transition_window = Widget::downcast_kind_mut::<TransitionWindow>(&window);
                transition_window.selected_transition = Some(index);
            }));

            let to = match &transition.to_area {
                &Some(ref to) => to.to_string(),
                &None => "".to_string(),
            };

            let text = format!("{}: {}", index, to);
            let entry = if self.selected_transition == Some(index) {
                list_box::Entry::with_active(text, Some(cb))
            } else {
                list_box::Entry::new(text, Some(cb))
            };

            entries.push(entry);
        }

        let transitions_box = Widget::with_theme(ListBox::new(entries), "transitions_list");
        widgets.push(transitions_box);

        let new = Widget::with_theme(Button::empty(), "new_button");

        let area_editor_ref = Rc::clone(&self.area_editor);
        new.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let window = Widget::get_parent(widget);
            window.borrow_mut().invalidate_children();

            let index = match area_editor_ref.borrow_mut().new_transition() {
                None => return,
                Some(index) => index,
            };

            let kind = &window.borrow().kind;
            let mut kind = kind.borrow_mut();
            let transition_window = match kind.as_any_mut().downcast_mut::<TransitionWindow>() {
                Some(window) => window,
                None => unreachable!("Unable to downcast to transition window."),
            };
            transition_window.selected_transition = Some(index);
        })));
        widgets.push(new);

        widgets
    }
}
