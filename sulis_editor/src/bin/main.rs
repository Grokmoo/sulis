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

extern crate sulis_core;
extern crate sulis_module;
extern crate sulis_state;
extern crate sulis_editor;

#[macro_use] extern crate log;

use std::rc::Rc;

use sulis_core::ui;
use sulis_core::config::Config;
use sulis_core::resource::ResourceSet;
use sulis_core::util;
use sulis_module::Module;

use sulis_editor::{EditorView, EditorMainLoopUpdater};

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    util::setup_logger();
    info!("Setup Logger and read configuration from 'config.yml'");

    let resources_config = Config::resources_config();

    let dir = resources_config.directory;
    info!("Reading resources from {}", dir);
    let data_dir = format!("../{}", dir);
    if let Err(e) = ResourceSet::init(&data_dir) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error reading resources.");
    };

    let campaigns_dir = resources_config.campaigns_directory;
    let module = Config::editor_config().module;
    info!("Reading module from {}", module);
    if let Err(e) =  Module::init(&data_dir, &format!("../{}/{}", campaigns_dir, module)) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error setting up the module.");
    };

    info!("Setting up display adapter.");
    let mut io = match sulis_core::io::create() {
        Ok(io) => io,
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("There was a fatal error initializing the display.");
            unreachable!();
        }
    };

    let root = ui::create_ui_tree(EditorView::new());
    match ResourceSet::get_theme().children.get("editor") {
        None => warn!("No theme found for 'editor'"),
        Some(ref theme) => {
            root.borrow_mut().theme = Some(Rc::clone(theme));
            root.borrow_mut().theme_id = ".editor".to_string();
            root.borrow_mut().theme_subname = "editor".to_string();
        }
    }

    if let Err(e) = util::main_loop(&mut io, root, Box::new(EditorMainLoopUpdater { })) {
        error!("{}", e);
        error!("Error in main loop.  Exiting...");
    }

    util::ok_and_exit("Main loop complete.");
}

