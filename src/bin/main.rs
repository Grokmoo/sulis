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

#![windows_subsystem = "windows"]

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
use sulis_module::{Actor};
use sulis_state::{GameState, NextGameStep, SaveState};
use sulis_view::{RootView, main_menu};

fn init() -> Box<IO> {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    util::setup_logger();
    info!("=========Initializing=========");
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

fn main_menu(io: &mut Box<IO>) -> NextGameStep {
    let view = main_menu::MainMenu::new();
    let loop_updater = main_menu::LoopUpdater::new(&view);
    let root = ui::create_ui_tree(view.clone());
    match ResourceSet::get_theme().children.get("main_menu") {
        None => warn!("No theme found for 'main_menu'"),
        Some(ref theme) => {
            root.borrow_mut().theme = Some(Rc::clone(theme));
            root.borrow_mut().theme_id = ".main_menu".to_string();
            root.borrow_mut().theme_subname = "main_menu".to_string();
        }
    }

    if let Err(e) = util::main_loop(io, root, Box::new(loop_updater)) {
        error!("{}", e);
        util::error_and_exit("Error in module starter.");
    }

    let mut view = view.borrow_mut();
    match view.next_step() {
        None => NextGameStep::Exit,
        Some(step) => step,
    }
}

fn new_campaign(io: &mut Box<IO>, pc_actor: Rc<Actor>) -> NextGameStep {
    info!("Initializing game state.");
    if let Err(e) = GameState::init(pc_actor) {
        error!("{}",  e);
        util::error_and_exit("There was a fatal error creating the game state.");
    };

    run_campaign(io)
}

fn load_campaign(io: &mut Box<IO>, save_state: SaveState) -> NextGameStep {
    info!("Loading game state.");
    if let Err(e) = GameState::load(save_state) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error loading the game state.");
    };

    run_campaign(io)
}

fn run_campaign(io: &mut Box<IO>) -> NextGameStep {
    let view = RootView::new();
    let loop_updater = sulis_view::GameMainLoopUpdater::new(&view);
    let root = ui::create_ui_tree(view.clone());

    if let Err(e) = util::main_loop(io, root, Box::new(loop_updater)) {
        error!("{}", e);
        error!("Error in main loop.  Exiting...");
    }

    let mut view = view.borrow_mut();
    match view.next_step() {
        None => NextGameStep::Exit,
        Some(step) => step,
    }
}

fn main() {
    let mut io = init();

    let mut next_step = NextGameStep::MainMenu;
    loop {
        next_step = match next_step {
            NextGameStep::Exit => break,
            NextGameStep::NewCampaign { pc_actor } => new_campaign(&mut io, pc_actor),
            NextGameStep::LoadCampaign { save_state } => load_campaign(&mut io, save_state),
            NextGameStep::MainMenu => main_menu(&mut io),
        };
    }

    util::ok_and_exit("Received exit result.");
}
