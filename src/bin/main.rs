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

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use log::{error, info};

use sulis_core::resource::ResourceSet;
use sulis_core::io::System;
use sulis_core::ui::{self, Cursor, Widget};
use sulis_core::util::{self, ActiveResources};
use sulis_module::{Actor, Module};
use sulis_state::{GameState, NextGameStep, SaveState};
use sulis_view::{main_menu::{self, MainMenu}, RootView, trigger_activator};

struct ControlFlowUpdater {
    root: Rc<RefCell<Widget>>,
    mode: UiMode,
    exit: bool,
}

#[derive(Clone)]
enum UiMode {
    MainMenu(Rc<RefCell<MainMenu>>),
    Game(Rc<RefCell<RootView>>),
}

impl ControlFlowUpdater {
    fn new(system: &System) -> ControlFlowUpdater {
        let view = main_menu::MainMenu::new(
            system.get_display_configurations(),
            sulis_core::io::audio::get_audio_devices(),
        );
        let root = ui::create_ui_tree(view.clone());

        ControlFlowUpdater {
            root,
            mode: UiMode::MainMenu(view),
            exit: false,
        }
    }

    fn handle_next_step(&mut self, step: NextGameStep) {
        use NextGameStep::*;
        match step {
            Exit => {
                self.exit = true;
            }, NewCampaign { pc_actor } => {

            }, LoadCampaign { save_state } => {

            }, LoadModuleAndNewCampaign { pc_actor, party_actors, flags, module_dir } => {

            }, MainMenu => {

            }, MainMenuReloadResources => {

            }, RecreateIo => {

            }
        }
    }

    fn update(&mut self, millis: u32) -> Rc<RefCell<Widget>> {
        let mode = self.mode.clone();

        match mode {
            UiMode::MainMenu(view) => {
                if let Some(step) = view.borrow_mut().next_step() {
                    self.handle_next_step(step);
                }
            }, UiMode::Game(view) => {
                let ui_cb = GameState::update(millis);

                if let Some(cb) = ui_cb {
                    trigger_activator::activate(&self.root, &cb.on_trigger, &cb.parent, &cb.target);
                }

                if let Some(step) = view.borrow_mut().next_step() {
                    self.handle_next_step(step);
                }
            }
        }

        Rc::clone(&self.root)
    }

    fn is_exit(&self) -> bool { self.exit }
}

fn init() -> System {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    util::setup_logger();
    info!("=========Initializing=========");
    info!("Setup Logger and read configuration from 'config.yml'");

    load_resources();

    create_io()
}

fn create_io() -> System {
    Cursor::update_max();

    info!("Setting up display adapter.");
    match System::create() {
        Ok(system) => system,
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("There was a fatal error initializing the display system.");
            unreachable!();
        }
    }
}

fn load_resources() {
    let active = ActiveResources::read();

    let dirs = active.directories();

    info!("Reading resources from '{:?}'", dirs);
    let yaml = match ResourceSet::load_resources(dirs.clone()) {
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("Fatal error reading resources.");
            unreachable!();
        }
        Ok(yaml) => yaml,
    };

    if dirs.len() > 1 {
        info!("Loading module '{}'", dirs[1]);
        match Module::load_resources(yaml, dirs) {
            Err(e) => {
                error!("{}", e);
            }
            Ok(()) => (),
        }
    }
}

fn main_menu(system: &mut System) -> NextGameStep {
    let view = main_menu::MainMenu::new(
        system.get_display_configurations(),
        sulis_core::io::audio::get_audio_devices(),
    );
    let loop_updater = main_menu::LoopUpdater::new(&view);
    let root = ui::create_ui_tree(view.clone());

    if let Err(e) = util::main_loop(system, root, Box::new(loop_updater)) {
        error!("{}", e);
        util::error_and_exit("Error in module starter.");
    }

    let mut view = view.borrow_mut();
    match view.next_step() {
        None => NextGameStep::Exit,
        Some(step) => step,
    }
}

fn new_campaign(
    system: &mut System,
    pc_actor: Rc<Actor>,
    party_actors: Vec<Rc<Actor>>,
    flags: HashMap<String, String>,
) -> NextGameStep {
    info!("Initializing game state.");
    if let Err(e) = GameState::init(pc_actor, party_actors, flags) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error creating the game state.");
    };

    run_campaign(system)
}

fn load_campaign(system: &mut System, save_state: SaveState) -> NextGameStep {
    info!("Loading game state.");
    if let Err(e) = GameState::load(save_state) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error loading the game state.");
    };

    run_campaign(system)
}

fn run_campaign(system: &mut System) -> NextGameStep {
    let view = RootView::new();
    let loop_updater = sulis_view::GameMainLoopUpdater::new(&view);
    let root = ui::create_ui_tree(view.clone());

    if let Err(e) = util::main_loop(system, root, Box::new(loop_updater)) {
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
    let mut system = init();

    let mut next_step = NextGameStep::MainMenu;
    loop {
        use sulis_state::NextGameStep::*;
        next_step = match next_step {
            Exit => break,
            NewCampaign { pc_actor } => {
                new_campaign(&mut system, pc_actor, Vec::new(), HashMap::new())
            }
            LoadCampaign { save_state } => load_campaign(&mut system, *save_state),
            MainMenu => main_menu(&mut system),
            MainMenuReloadResources => {
                load_resources();
                main_menu(&mut system)
            }
            LoadModuleAndNewCampaign {
                pc_actor,
                party_actors,
                flags,
                module_dir,
            } => {
                let mut active = ActiveResources::read();
                active.campaign = Some(module_dir);
                active.write();
                load_resources();
                new_campaign(&mut system, pc_actor, party_actors, flags)
            }
            RecreateIO => {
                system = create_io();
                main_menu(&mut system)
            }
        };
    }

    util::ok_and_exit("Received exit result.");
}
