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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::area::MAX_AREA_SIZE;
use sulis_widgets::{Button, InputField, Label, list_box, ListBox, Spinner};

use AreaEditor;

pub const NAME: &str = "transition_window";

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
            let transition = area_editor.model.transition(index);

            let to_area_str = match transition.to_area {
                None => "",
                Some(ref area) => area,
            };

            let max = MAX_AREA_SIZE - 1;
            let from_x = Spinner::new(transition.from.x, 0, max);
            let from_y = Spinner::new(transition.from.y, 0, max);

            let to_x = Spinner::new(transition.to.x, 0, max);
            let to_y = Spinner::new(transition.to.y, 0, max);

            let to_area = Widget::with_theme(InputField::new(to_area_str), "to_area");
            let from_label = Widget::with_theme(Label::empty(), "from_label");
            let to_label = Widget::with_theme(Label::empty(), "to_label");
            let to_area_label = Widget::with_theme(Label::empty(), "to_area_label");

            let apply = Widget::with_theme(Button::empty(), "apply_button");

            let to_area_ref = Rc::clone(&to_area);
            let area_editor_ref = Rc::clone(&self.area_editor);

            let from_x_ref = Rc::clone(&from_x);
            let from_y_ref = Rc::clone(&from_y);
            let to_x_ref = Rc::clone(&to_x);
            let to_y_ref = Rc::clone(&to_y);
            apply.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let to_area_str = to_area_ref.borrow().state.text.to_string();
                let to_area = if to_area_str.is_empty() {
                    None
                } else {
                    Some(to_area_str)
                };

                let from = Point::new(from_x_ref.borrow().value(), from_y_ref.borrow().value());
                let to = Point::new(to_x_ref.borrow().value(), to_y_ref.borrow().value());

                let window = Widget::get_parent(widget);
                window.borrow_mut().invalidate_children();

                let transition_window = Widget::downcast_kind_mut::<TransitionWindow>(&window);
                let cur_index = match transition_window.selected_transition {
                    Some(index) => index,
                    None => return,
                };

                let mut area_editor = area_editor_ref.borrow_mut();
                let transition = area_editor.model.transition_mut(cur_index);
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

                area_editor_ref.borrow_mut().model.delete_transition(cur_index);
                transition_window.selected_transition = None;
            })));

            widgets.append(&mut vec![to_area, from_label, to_label,
                           to_area_label, apply, delete]);
            widgets.append(&mut vec![Widget::with_theme(to_x, "to_x"), Widget::with_theme(to_y, "to_y"),
                Widget::with_theme(from_x, "from_x"), Widget::with_theme(from_y, "from_y")]);
        }

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, ref transition) in self.area_editor.borrow().model.transitions_iter().enumerate() {
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

            let index = match area_editor_ref.borrow_mut().model.new_transition() {
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
