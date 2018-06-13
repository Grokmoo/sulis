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
use std::path::{PathBuf};
use std::fs;
use std::io::Error;

use sulis_core::config;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::resource::write_to_file;
use sulis_widgets::{Button, ConfirmationWindow, Label};
use sulis_module::Module;

const NAME: &str = "save_window";

fn get_save_dir() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("save");
    path.push(&Module::game().id);
    path
}

pub struct SaveWindow {
    accept: Rc<RefCell<Widget>>,
    entries: Vec<PathBuf>,
}

impl SaveWindow {
    pub fn new() -> Rc<RefCell<SaveWindow>> {
        let accept = Widget::with_theme(Button::empty(), "accept");
        let entries = match SaveWindow::get_available_files() {
            Ok(files) => files,
            Err(e) => {
                warn!("Unable to read saved files");
                warn!("{}", e);
                Vec::new()
            }
        };

        Rc::new(RefCell::new(SaveWindow { accept, entries }))
    }

    fn get_available_files() -> Result<Vec<PathBuf>, Error> {
        let mut results: Vec<PathBuf> = Vec::new();

        let dir = get_save_dir();
        debug!("Reading save games from {}", dir.to_string_lossy());

        if !dir.is_dir() {
            fs::create_dir_all(dir.clone())?;
        }

        let dir_entries = fs::read_dir(dir)?;

        for entry in dir_entries {
            trace!("Checking entry {:?}", entry);
            let entry = entry?;

            let path = entry.path();
            if !path.is_file() { continue; }

            let extension = match path.extension() {
                None => continue,
                Some(ext) => ext.to_string_lossy(),
            };

            if extension != "yml" { continue; }

            results.push(path.to_path_buf());
        }

        Ok(results)
    }

    pub fn save(&self) {

    }
}

impl WidgetKind for SaveWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();
        })));

        self.accept.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let save_window = Widget::downcast_kind_mut::<SaveWindow>(&parent);
            save_window.save();

            parent.borrow_mut().mark_for_removal();
        })));
        self.accept.borrow_mut().state.set_enabled(false);

        let entries = Widget::empty("entries");

        for path_buf in self.entries.iter() {
            let label = path_buf.to_string_lossy().to_string();
            let widget = Widget::with_theme(Button::with_text(&label), "entry");
            Widget::add_child_to(&entries, widget);
        }

        vec![cancel, self.accept.clone(), title, entries]
    }
}
