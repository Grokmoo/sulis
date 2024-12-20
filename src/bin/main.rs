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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{error, info};

use sulis_core::io::{ControlFlowUpdater, DisplayConfiguration, System};
use sulis_core::resource::ResourceSet;
use sulis_core::ui::{self, Cursor, Widget};
use sulis_core::util::{self, ActiveResources};
use sulis_module::{Actor, Module};
use sulis_state::{GameState, NextGameStep, SaveState};
use sulis_view::{
    main_menu::{self, MainMenu},
    trigger_activator, RootView,
};

struct GameControlFlowUpdater {
    display_configurations: Vec<DisplayConfiguration>,
    recreate_window: bool,

    root: Rc<RefCell<Widget>>,
    mode: UiMode,
    exit: bool,

    next_step: Option<NextGameStep>,
}

#[derive(Clone)]
enum UiMode {
    MainMenu(Rc<RefCell<MainMenu>>),
    Game(Rc<RefCell<RootView>>),
}

impl ControlFlowUpdater for GameControlFlowUpdater {
    fn update(&mut self, millis: u32) -> Rc<RefCell<Widget>> {
        if let Some(step) = self.next_step.take() {
            self.handle_next_step(step);
        }

        self.update_mode(millis);

        if let Err(e) = Widget::update(&self.root, millis) {
            error!("There was a fatal error updating the UI tree state.");
            error!("{}", e);
            self.exit = true;
            return self.root();
        }

        self.root()
    }

    fn recreate_window(&mut self) -> bool {
        let recreate = self.recreate_window;
        self.recreate_window = false;
        recreate
    }

    fn root(&self) -> Rc<RefCell<Widget>> {
        Rc::clone(&self.root)
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}

impl GameControlFlowUpdater {
    fn new(system: &System) -> GameControlFlowUpdater {
        let display_configurations = system.get_display_configurations();
        let view = main_menu::MainMenu::new(
            display_configurations.clone(),
            sulis_core::io::audio::get_audio_devices(),
        );
        let root = ui::create_ui_tree(view.clone());

        GameControlFlowUpdater {
            display_configurations,
            recreate_window: false,
            root,
            mode: UiMode::MainMenu(view),
            exit: false,
            next_step: None,
        }
    }

    fn main_menu(&mut self) {
        let view = main_menu::MainMenu::new(
            self.display_configurations.clone(),
            sulis_core::io::audio::get_audio_devices(),
        );
        self.root = ui::create_ui_tree(view.clone());
        self.mode = UiMode::MainMenu(view);
    }

    fn new_campaign(
        &mut self,
        pc_actor: Rc<Actor>,
        party_actors: Vec<Rc<Actor>>,
        flags: HashMap<String, String>,
    ) {
        info!("Initializing game state.");
        if let Err(e) = GameState::init(pc_actor, party_actors, flags) {
            error!("{}", e);
            util::error_and_exit("There was a fatal error creating the game state.");
        };

        let view = RootView::new();
        self.root = ui::create_ui_tree(view.clone());
        self.mode = UiMode::Game(view);
    }

    fn load_campaign(&mut self, save_state: SaveState) {
        info!("Loading game state.");
        if let Err(e) = GameState::load(save_state) {
            error!("{}", e);
            util::error_and_exit("There was a fatal error loading the game state.");
        };

        let view = RootView::new();
        self.root = ui::create_ui_tree(view.clone());
        self.mode = UiMode::Game(view);
    }

    fn handle_next_step(&mut self, step: NextGameStep) {
        use NextGameStep::*;
        match step {
            Exit => {
                self.exit = true;
            }
            NewCampaign { pc_actor } => {
                self.new_campaign(pc_actor, Vec::new(), HashMap::new());
            }
            LoadCampaign { save_state } => {
                self.load_campaign(*save_state);
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
                self.new_campaign(pc_actor, party_actors, flags);
            }
            MainMenu => {
                self.main_menu();
            }
            MainMenuReloadResources => {
                load_resources();
                self.main_menu();
            }
            RecreateIO => {
                self.recreate_window = true;
                self.main_menu();
            }
        }
    }

    fn update_mode(&mut self, millis: u32) {
        let mode = self.mode.clone();

        // here, save the next step to be handled next frame to give a
        // chance for the loading screen to pop up
        match mode {
            UiMode::MainMenu(view) => {
                self.next_step = view.borrow_mut().next_step();
            }
            UiMode::Game(view) => {
                let ui_cb = GameState::update(millis);

                if let Some(cb) = ui_cb {
                    trigger_activator::activate(&self.root, &cb.on_trigger, &cb.parent, &cb.target);
                }

                self.next_step = view.borrow_mut().next_step();
            }
        }
    }
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
    let start = std::time::Instant::now();

    let active = ActiveResources::read();

    let dirs = active.directories();

    let start_main = std::time::Instant::now();
    info!("Reading resources from '{:?}'", dirs);
    let yaml = match ResourceSet::load_resources(dirs.clone()) {
        Err(e) => {
            error!("{}", e);
            util::error_and_exit("Fatal error reading resources.");
            unreachable!();
        }
        Ok(yaml) => yaml,
    };
    info!(
        "Loaded base resources in {}s",
        util::format_elapsed_secs(start_main.elapsed())
    );

    if dirs.len() > 1 {
        info!("Loading module '{}'", dirs[1]);
        if let Err(e) = Module::load_resources(yaml, dirs) {
            error!("{}", e);
        }
    }

    info!(
        "Loaded all resources in {}s",
        util::format_elapsed_secs(start.elapsed())
    );
}

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits.  Don't drop the returned handle
    // while the program is running
    let _logger_handle = util::setup_logger();
    info!("=========Initializing=========");
    info!("Setup Logger and read configuration from 'config.yml'");

    load_resources();

    let system = create_io();

    let flow_controller = GameControlFlowUpdater::new(&system);
    system.main_loop(Box::new(flow_controller));
}
