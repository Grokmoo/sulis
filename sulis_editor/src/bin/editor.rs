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

#[macro_use]
extern crate log;

use sulis_core::config::Config;
use sulis_core::resource::ResourceSet;
use sulis_core::ui;
use sulis_core::util;
use sulis_core::io::{glium_adapter};
use sulis_module::Module;

use sulis_editor::{EditorMainLoopUpdater, EditorView};

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    util::setup_logger();
    info!("Setup Logger and read configuration from 'config.yml'");

    let resources_config = Config::resources_config();

    let dir = resources_config.directory;
    let data_dir = format!("../{}", dir);

    let campaigns_dir = resources_config.campaigns_directory;
    let module = Config::editor_config().module;
    let module_dir = format!("../{}/{}", campaigns_dir, module);

    let dirs = vec![data_dir, module_dir];
    info!("Reading resources from {:?}", dirs);

    let yaml = match ResourceSet::load_resources(dirs.clone()) {
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("Fatal error reading resources.");
            unreachable!();
        }
        Ok(yaml) => yaml,
    };

    if let Err(e) = Module::load_resources(yaml, dirs) {
        error!("{}", e);
        util::error_and_exit("Fatal error settign up module.");
    }

    info!("Setting up display adapter.");
    let (io, event_loop, audio) = match sulis_core::io::create_glium() {
        Ok(data) => data,
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("Fatal error initializing the display.");
            unreachable!();
        }
    };

    let root = ui::create_ui_tree(EditorView::new());

    glium_adapter::main_loop(io, event_loop, audio, Box::new(EditorMainLoopUpdater {}), root);
}

