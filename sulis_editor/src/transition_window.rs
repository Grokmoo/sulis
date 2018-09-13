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
use sulis_module::area::{ToKind, MAX_AREA_SIZE};
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


            let (to_area_str, to_x, to_y) = match transition.to {
                ToKind::CurArea { x, y } => (String::new(), x, y),
                ToKind::Area { ref id, x, y } => (id.to_string(), x, y),
                ToKind::WorldMap => (String::new(), 0, 0),
            };

            let max = MAX_AREA_SIZE - 1;
            let from_x = Spinner::new(transition.from.x, 0, max);
            let from_y = Spinner::new(transition.from.y, 0, max);

            let to_x = Spinner::new(to_x, 0, max);
            let to_y = Spinner::new(to_y, 0, max);

            let to_area = Widget::with_theme(InputField::new(&to_area_str), "to_area");
            let from_label = Widget::with_theme(Label::empty(), "from_label");
            let to_label = Widget::with_theme(Label::empty(), "to_label");
            let to_area_label = Widget::with_theme(Label::empty(), "to_area_label");

            let apply = Widget::with_theme(Button::empty(), "apply_button");

            let to_area_ref = Rc::clone(&to_area);

            let from_x_ref = Rc::clone(&from_x);
            let from_y_ref = Rc::clone(&from_y);
            let to_x_ref = Rc::clone(&to_x);
            let to_y_ref = Rc::clone(&to_y);

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

            let cur_area_button = Widget::with_theme(Button::empty(), "cur_area_button");
            let area_button = Widget::with_theme(Button::empty(), "area_button");
            let world_map_button = Widget::with_theme(Button::empty(), "world_map_button");

            match transition.to {
                ToKind::CurArea { .. } => cur_area_button.borrow_mut().state.set_active(true),
                ToKind::Area { .. } => area_button.borrow_mut().state.set_active(true),
                ToKind::WorldMap => world_map_button.borrow_mut().state.set_active(true),
            }

            let refs = vec![Rc::clone(&cur_area_button), Rc::clone(&area_button),
                Rc::clone(&world_map_button)];
            let refs_clone = refs.clone();
            for widget in refs {
                let widget_refs = refs_clone.clone();
                widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move | widget, _| {
                    for widget in widget_refs.iter() {
                        widget.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);
                })));
            }

            let area_editor_ref = Rc::clone(&self.area_editor);
            let world_map_ref = Rc::clone(&world_map_button);
            let area_ref = Rc::clone(&area_button);
            let cur_area_ref = Rc::clone(&cur_area_button);
            apply.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let to_area_str = to_area_ref.borrow().state.text.to_string();

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

                if world_map_ref.borrow().state.is_active() {
                    transition.to = ToKind::WorldMap;
                } else if area_ref.borrow().state.is_active() {
                    transition.to = ToKind::Area { id: to_area_str, x: to.x, y: to.y };
                } else if cur_area_ref.borrow().state.is_active() {
                    transition.to = ToKind::CurArea { x: to.x, y: to.y };
                }
            })));


            widgets.append(&mut vec![cur_area_button, area_button, world_map_button]);

            widgets.append(&mut vec![to_area, from_label, to_label, to_area_label, apply, delete]);
            widgets.append(&mut vec![Widget::with_theme(to_x, "to_x"),
                Widget::with_theme(to_y, "to_y"),
                Widget::with_theme(from_x, "from_x"),
                Widget::with_theme(from_y, "from_y")]);
        }

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, ref transition) in self.area_editor.borrow().model.transitions_iter().enumerate() {
            let cb = Callback::new(Rc::new(move |widget, _| {
                let window = Widget::go_up_tree(widget, 2);
                window.borrow_mut().invalidate_children();

                let transition_window = Widget::downcast_kind_mut::<TransitionWindow>(&window);
                transition_window.selected_transition = Some(index);
            }));

            let to = match transition.to {
                ToKind::CurArea { .. } => "self".to_string(),
                ToKind::Area { ref id, .. } => id.to_string(),
                ToKind::WorldMap => "World Map".to_string(),
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
