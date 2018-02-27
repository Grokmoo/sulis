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
extern crate sulis_view;

#[macro_use] extern crate log;

use std::rc::Rc;

use sulis_core::ui;
use sulis_core::config::CONFIG;
use sulis_core::resource::ResourceSet;
use sulis_core::util;
use sulis_module::Module;
use sulis_state::{GameStateMainLoopUpdater, GameState};
use sulis_view::RootView;
use sulis_view::character_selector::{self, CharacterSelector};
use sulis_view::main_menu::{MainMenuView, MainMenuLoopUpdater};

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    util::setup_logger();
    info!("Setup Logger and read configuration from 'config.yml'");

    info!("Reading resources from {}", CONFIG.resources.directory);
    if let Err(e) = ResourceSet::init(&CONFIG.resources.directory) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error reading resources..");
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

    let modules_list = Module::get_available_modules("modules");
    if modules_list.len() == 0 {
        util::error_and_exit("No valid modules found.");
    }

    let selected_module = {
        let main_menu_view = MainMenuView::new(modules_list);
        let loop_updater = MainMenuLoopUpdater::new(&main_menu_view);
        let main_menu_root = ui::create_ui_tree(main_menu_view.clone());
        match ResourceSet::get_theme().children.get("main_menu") {
            None => warn!("No theme found for 'main_menu"),
            Some(ref theme) => {
                main_menu_root.borrow_mut().theme = Some(Rc::clone(theme));
                main_menu_root.borrow_mut().theme_id = ".main_menu".to_string();
                main_menu_root.borrow_mut().theme_subname = "main_menu".to_string();
            }
        }

        if let Err(e) = util::main_loop(&mut io, main_menu_root, Box::new(loop_updater)) {
            error!("{}", e);
            util::error_and_exit("Error in main menu.");
        }

        let mmv_ref = main_menu_view.borrow();
        mmv_ref.get_selected_module()
    };

    let module_info = match selected_module {
        None => {
            util::ok_and_exit("No module selected in main menu.");
            unreachable!();
        },
        Some(module) => module,
    };

    info!("Reading module from {}", module_info.dir);
    if let Err(e) =  Module::init(&CONFIG.resources.directory, &module_info.dir) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error setting up the module.");
    };

    let pc_actor = {
        let view = CharacterSelector::new();
        let loop_updater = character_selector::LoopUpdater::new(&view);
        let root = ui::create_ui_tree(view.clone());
        match ResourceSet::get_theme().children.get("character_selector") {
            None => warn!("No theme found for 'character_selector'"),
            Some(ref theme) => {
                root.borrow_mut().theme = Some(Rc::clone(theme));
                root.borrow_mut().theme_id = ".character_selector".to_string();
                root.borrow_mut().theme_subname = "character_selector".to_string();
            }
        }

        if let Err(e) = util::main_loop(&mut io, root, Box::new(loop_updater)) {
            error!("{}", e);
            util::error_and_exit("Error in character selector.");
        }

        let view = view.borrow();
        view.selected()
    };

    let pc_actor = match pc_actor {
        None => {
            util::ok_and_exit("No actor selected in main menu.");
            unreachable!();
        },
        Some(actor) => actor,
    };

    info!("Initializing game state.");
    if let Err(e) = GameState::init(pc_actor) {
        error!("{}",  e);
        util::error_and_exit("There was a fatal error creating the game state.");
    };

    let root = ui::create_ui_tree(RootView::new());

    if let Err(e) = util::main_loop(&mut io, root, Box::new(GameStateMainLoopUpdater { })) {
        error!("{}", e);
        error!("Error in main loop.  Exiting...");
    }

    util::ok_and_exit("Main loop complete.");
}
