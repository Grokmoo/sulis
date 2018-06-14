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

use sulis_core::{config, serde_yaml};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::resource::{ResourceBuilder, read_single_resource_path};
use sulis_core::util::{invalid_data_error};
use sulis_widgets::{Button, Label, TextArea};
use sulis_module::Module;

const NAME: &str = "load_window";

fn get_save_dir() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("save");
    path.push(&Module::game().id);
    path
}

pub struct LoadWindow {
    accept: Rc<RefCell<Widget>>,
    entries: Vec<(PathBuf, SaveFileMetaData)>,
    selected_entry: Option<usize>,
}

impl LoadWindow {
    pub fn new() -> Rc<RefCell<LoadWindow>> {
        let accept = Widget::with_theme(Button::empty(), "accept");
        let entries = match LoadWindow::get_available_files() {
            Ok(files) => files,
            Err(e) => {
                warn!("Unable to read saved files");
                warn!("{}", e);
                Vec::new()
            }
        };

        Rc::new(RefCell::new(LoadWindow { accept, entries, selected_entry: None }))
    }

    fn get_available_files() -> Result<Vec<(PathBuf, SaveFileMetaData)>, Error> {
        let mut results: Vec<(PathBuf, SaveFileMetaData)> = Vec::new();

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

            let path_buf = path.to_path_buf();

            let save_file: SaveFile = read_single_resource_path(&path_buf)?;

            results.push((path_buf, save_file.meta));
        }

        Ok(results)
    }

    pub fn load(&self) {
        // TODO implement load
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

        self.accept.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let load_window = Widget::downcast_kind_mut::<LoadWindow>(&parent);
            load_window.load();

            parent.borrow_mut().mark_for_removal();
        })));
        self.accept.borrow_mut().state.set_enabled(self.selected_entry.is_some());

        let entries = Widget::empty("entries");

        for (index, (ref _path_buf, ref meta)) in self.entries.iter().enumerate() {
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

        vec![cancel, self.accept.clone(), title, entries]
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    meta: SaveFileMetaData,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFileMetaData {
    player_name: String,
    datetime: String,
    current_area_name: String,
}

impl ResourceBuilder for SaveFile {
    fn owned_id(&self) -> String {
        self.meta.player_name.to_string()
    }

    fn from_yaml(data: &str) -> Result<Self, Error> {
        let resource: Result<SaveFile, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
