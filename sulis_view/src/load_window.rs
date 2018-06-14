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
use sulis_widgets::{Button, Label, TextArea, ConfirmationWindow};
use sulis_state::{SaveFileMetaData, save_file::{delete_save, load_state, get_available_save_files}};

const NAME: &str = "load_window";

pub struct LoadWindow {
    accept: Rc<RefCell<Widget>>,
    entries: Vec<SaveFileMetaData>,
    selected_entry: Option<usize>,
}

impl LoadWindow {
    pub fn new() -> Rc<RefCell<LoadWindow>> {
        let accept = Widget::with_theme(Button::empty(), "accept");
        let entries = match get_available_save_files() {
            Ok(files) => files,
            Err(e) => {
                warn!("Unable to read saved files");
                warn!("{}", e);
                Vec::new()
            }
        };

        Rc::new(RefCell::new(LoadWindow { accept, entries, selected_entry: None }))
    }

    pub fn load(&self) {
        let index = match self.selected_entry {
            None => return,
            Some(index) => index,
        };

        match load_state(&self.entries[index]) {
            Err(e) => {
                error!("Error loading game state");
                error!("{}", e);
            }, Ok(()) => (),
        }
    }

    pub fn delete_save(&mut self) {
        let index = match self.selected_entry {
            None => return,
            Some(index) => index,
        };

        match delete_save(&self.entries[index]) {
            Err(e) => {
                error!("Error deleting save");
                error!("{}", e);
            }, Ok(()) => (),
        }

        self.entries.remove(index);
    }
}

impl WidgetKind for LoadWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();
        })));

        let load_window_widget_ref = Rc::clone(widget);
        let delete_cb = Callback::new(Rc::new(move |widget, _| {
            load_window_widget_ref.borrow_mut().invalidate_children();

            let load_window = Widget::downcast_kind_mut::<LoadWindow>(&load_window_widget_ref);
            load_window.delete_save();
            load_window.selected_entry = None;

            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();
        }));

        let delete = Widget::with_theme(Button::empty(), "delete");
        delete.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let root = Widget::get_root(widget);
            let conf_window = Widget::with_theme(ConfirmationWindow::new(delete_cb.clone()), "delete_save_confirmation");
            conf_window.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, conf_window);
        })));
        delete.borrow_mut().state.set_enabled(self.selected_entry.is_some());

        self.accept.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let load_window = Widget::downcast_kind_mut::<LoadWindow>(&parent);
            load_window.load();

            parent.borrow_mut().mark_for_removal();
        })));
        self.accept.borrow_mut().state.set_enabled(self.selected_entry.is_some());

        let entries = Widget::empty("entries");

        for (index, ref meta) in self.entries.iter().enumerate() {
            let text_area = Widget::with_defaults(TextArea::empty());
            text_area.borrow_mut().state.add_text_arg("player_name", &meta.player_name);
            text_area.borrow_mut().state.add_text_arg("datetime", &meta.datetime);
            text_area.borrow_mut().state.add_text_arg("current_area_name", &meta.current_area_name);
            let widget = Widget::with_theme(Button::empty(), "entry");
            widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 2);
                parent.borrow_mut().invalidate_children();

                let load_window = Widget::downcast_kind_mut::<LoadWindow>(&parent);
                load_window.selected_entry = Some(index);
            })));

            if let Some(selected_index) = self.selected_entry {
                if selected_index == index {
                    widget.borrow_mut().state.set_active(true);
                }
            }

            Widget::add_child_to(&widget, text_area);

            Widget::add_child_to(&entries, widget);
        }

        vec![cancel, delete, self.accept.clone(), title, entries]
    }
}
