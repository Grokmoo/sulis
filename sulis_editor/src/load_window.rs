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

use std::fs;
use std::ffi::OsStr;
use std::path::Path;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::config::Config;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, list_box, ListBox, ScrollPane};

use crate::AreaEditor;

pub const NAME: &str = "load_window";

pub struct LoadWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
}

impl LoadWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<LoadWindow>> {
        Rc::new(RefCell::new(LoadWindow {
            area_editor
        }))
    }
}

impl WidgetKind for LoadWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let (parent, _) = Widget::parent_mut::<LoadWindow>(widget);
            parent.borrow_mut().mark_for_removal();
        })));

        let load = Widget::with_theme(Button::empty(), "load_button");
        load.borrow_mut().state.set_enabled(false);

        let campaigns_dir = Config::resources_config().campaigns_directory;
        let dir_str = format!("../{}/{}/areas/", campaigns_dir, Config::editor_config().module);
        let areas = get_area_entries(&dir_str);

        let load_ref = Rc::clone(&load);
        let cb = Callback::new(Rc::new(move |widget, _kind| {
            let parent = Widget::direct_parent(widget);

            let cur_state = widget.borrow().state.is_active();
            if !cur_state {
                for child in parent.borrow().children.iter() {
                    child.borrow_mut().state.set_active(false);
                }
            }
            load_ref.borrow_mut().state.set_enabled(!cur_state);
            widget.borrow_mut().state.set_active(!cur_state);
        }));

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for area in areas {
            let entry = list_box::Entry::new(area, Some(cb.clone()));
            entries.push(entry);
        }
        let scrollpane = ScrollPane::new();
        let areas_list = Widget::with_theme(ListBox::new(entries), "listbox");

        let area_editor_ref = Rc::clone(&self.area_editor);
        let areas_list_ref = Rc::clone(&areas_list);
        load.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let areas = areas_list_ref.borrow();
            let active_child = match areas.children.iter().find(|c| c.borrow().state.is_active()) {
                None => return,
                Some(child) => child,
            };
            let area = &active_child.borrow().state.text;
            info!("Selected area to load: {}", area);

            area_editor_ref.borrow_mut().model.load(&dir_str, area);

            let (parent, _) = Widget::parent::<LoadWindow>(widget);
            parent.borrow_mut().mark_for_removal();
        })));
        scrollpane.borrow().add_to_content(areas_list);

        vec![close, load, Widget::with_theme(scrollpane, "areas_list")]
    }
}

fn get_area_entries(dir_str: &str) -> Vec<String> {
    let mut areas: Vec<String> = Vec::new();

    debug!("Reading area files from {}", dir_str);
    let dir = Path::new(&dir_str);

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => {
            warn!("Unable to read areas from directory {}", dir_str);
            return areas;
        }
    };

    for entry in dir_entries {
        trace!("Found entry{:?}", entry);
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading file: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension: String = OsStr::to_str(path.extension().unwrap_or(
                OsStr::new(""))).unwrap_or("").to_string();

        if &extension != "yml" {
            continue;
        }

        match OsStr::to_str(path.file_stem().unwrap_or(OsStr::new(""))) {
            None => (),
            Some(filename) => areas.push(filename.to_string()),
        }
    }

    areas
}
