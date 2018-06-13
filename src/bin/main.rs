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
use sulis_core::io::IO;
use sulis_core::util;
use sulis_module::{Actor, Module};
use sulis_state::{GameState, NextGameStep};
use sulis_view::{RootView};
use sulis_view::character_selector::{self, CharacterSelector};
use sulis_view::main_menu::{MainMenuView, MainMenuLoopUpdater};

fn init() -> Box<IO> {
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
    match sulis_core::io::create() {
        Ok(io) => io,
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("There was a fatal error initializing the display.");
            unreachable!();
        }
    }
}

fn select_module(io: &mut Box<IO>) -> NextGameStep {
    let modules_list = Module::get_available_modules("modules");
    if modules_list.len() == 0 {
        util::error_and_exit("No valid modules found.");
    }

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

    if let Err(e) = util::main_loop(io, main_menu_root, Box::new(loop_updater)) {
        error!("{}", e);
        util::error_and_exit("Error in main menu.");
    }

    let mmv_ref = main_menu_view.borrow();
    match mmv_ref.get_selected_module() {
        None => {
            NextGameStep::Exit
        },
        Some(module) => {
            info!("Reading module from {}", module.dir);
            if let Err(e) =  Module::init(&CONFIG.resources.directory, &module.dir) {
                error!("{}", e);
                util::error_and_exit("There was a fatal error setting up the module.");
            };
            NextGameStep::SelectCharacter
        }
    }
}

fn select_character(io: &mut Box<IO>) -> NextGameStep {
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

    if let Err(e) = util::main_loop(io, root, Box::new(loop_updater)) {
        error!("{}", e);
        util::error_and_exit("Error in character selector.");
    }

    let view = view.borrow();
    match view.next_step() {
        None => NextGameStep::Exit,
        Some(step) => step,
    }
}

fn run_campaign(io: &mut Box<IO>, pc_actor: Rc<Actor>) -> NextGameStep {
    info!("Initializing game state.");
    if let Err(e) = GameState::init(pc_actor) {
        error!("{}",  e);
        util::error_and_exit("There was a fatal error creating the game state.");
    };

    let view = RootView::new();
    let loop_updater = sulis_view::GameMainLoopUpdater::new(&view);
    let root = ui::create_ui_tree(view.clone());

    if let Err(e) = util::main_loop(io, root, Box::new(loop_updater)) {
        error!("{}", e);
        error!("Error in main loop.  Exiting...");
    }

    let view = view.borrow();
    match view.next_step() {
        None => NextGameStep::Exit,
        Some(step) => step,
    }
}

fn main() {
    let mut io = init();

    let mut next_step = NextGameStep::SelectModule;
    loop {
        next_step = match next_step {
            NextGameStep::Exit => break,
            NextGameStep::PlayCampaign { pc_actor } => run_campaign(&mut io, pc_actor),
            NextGameStep::SelectCharacter => select_character(&mut io),
            NextGameStep::SelectModule => select_module(&mut io),
        };
    }

    util::ok_and_exit("Received exit result.");
}
