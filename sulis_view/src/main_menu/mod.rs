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

pub mod character_selector;
pub use self::character_selector::CharacterSelector;

mod links_pane;
use self::links_pane::LinksPane;

pub mod module_selector;
pub use self::module_selector::ModuleSelector;

pub mod options;
pub use self::options::Options;

use std::fs::File;
use std::io::{prelude::*, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::any::Any;
use std::rc::Rc;
use std::cell::{RefCell};

use sulis_core::config::{self, Config};
use sulis_core::resource::ResourceSet;
use sulis_core::io::{InputAction, MainLoopUpdater, DisplayConfiguration};
use sulis_core::ui::*;
use sulis_core::util;
use sulis_widgets::{Button, ConfirmationWindow, Label};
use sulis_module::{Module};
use sulis_state::{NextGameStep, save_file};

use {CharacterBuilder, LoadWindow};

pub struct LoopUpdater {
    view: Rc<RefCell<MainMenu>>,
}

impl LoopUpdater {
    pub fn new(view: &Rc<RefCell<MainMenu>>) -> LoopUpdater {
        LoopUpdater {
            view: Rc::clone(view),
        }
    }
}

impl MainLoopUpdater for LoopUpdater {
    fn update(&self, _root: &Rc<RefCell<Widget>>, _millis: u32) { }

    fn is_exit(&self) -> bool {
        self.view.borrow().is_exit()
    }
}

enum Mode {
    New,
    Load,
    Module,
    Options,
    Links,
    NoChoice,
}

fn selected_module_file_path() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("selected_module.txt");
    path
}

fn write_selected_module_file(module_dir: &str) -> Result<(), Error> {
    let mut file = File::create(selected_module_file_path())?;
    file.write_all(module_dir.as_bytes())?;

    Ok(())
}

fn check_selected_module_file() -> Result<String, Error> {
    let mut file = File::open(selected_module_file_path())?;
    let mut module_dir = String::new();
    file.read_to_string(&mut module_dir)?;
    module_dir.trim();

    let campaign_file = format!("{}/campaign.yml", module_dir);
    let campaign_path = Path::new(&campaign_file);
    if !campaign_path.is_file() {
        warn!("Selected module file does not point to a valid campaign");
        return Err(Error::new(ErrorKind::InvalidData, "Invalid selected_module.txt file"));
    }

    Ok(module_dir)
}

fn load_module_internal(module_dir: &str) {
    let reload_resources = match Module::module_dir() {
        None => false,
        Some(ref dir) => module_dir != dir,
    };

    if reload_resources {
        // reload resources if we have already loaded a module previously
        // to prevent resources from the old module persisting

        let resources_dir = Config::resources_config().directory;
        info!("Reading resources from {}", resources_dir);
        if let Err(e) = ResourceSet::init(&resources_dir) {
            error!("{}", e);
            util::error_and_exit("There was a fatal error reading resources..");
        };
    }

    info!("Reading module from {}", module_dir);
    if let Err(e) =  Module::init(&Config::resources_config().directory, module_dir) {
        error!("{}", e);
        util::error_and_exit("There was a fatal error setting up the module.");
    };
}

pub fn load_module(module_dir: &str) {
    load_module_internal(module_dir);

    match write_selected_module_file(module_dir) {
        Ok(()) => (),
        Err(e) => {
            warn!("Unable to write selected module file");
            warn!("{}", e);
        }
    }
}

pub struct MainMenu {
    pub(crate) next_step: Option<NextGameStep>,
    mode: Mode,
    content: Rc<RefCell<Widget>>,
    pub(crate) char_builder_to_add: Option<Rc<RefCell<CharacterBuilder>>>,
    pub(crate) display_configurations: Vec<DisplayConfiguration>,
}

impl MainMenu {
    pub fn new(display_configurations: Vec<DisplayConfiguration>) -> Rc<RefCell<MainMenu>> {
        match check_selected_module_file() {
            Ok(dir) => load_module_internal(&dir),
            Err(e) => {
                info!("Unable to read selected_module file");
                info!("{}", e);
            }
        }

        Rc::new(RefCell::new(MainMenu {
            next_step: None,
            mode: Mode::NoChoice,
            content: Widget::empty("content"),
            char_builder_to_add: None,
            display_configurations,
        }))
    }

    pub fn reset(&mut self) {
        self.mode = Mode::NoChoice;
        self.content = Widget::empty("content");
    }

    pub fn is_exit(&self) -> bool {
        self.next_step.is_some()
    }

    pub fn next_step(&mut self) -> Option<NextGameStep> {
        self.next_step.take()
    }

    pub fn recreate_io(&mut self) {
        self.next_step = Some(NextGameStep::RecreateIO);
    }

    pub fn load_module(&mut self, module_dir: &str) {
        load_module(module_dir);

        self.reset();
    }
}

impl WidgetKind for MainMenu {
    widget_kind!("main_menu");

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            Exit | ShowMenu => {
                let exit_window = Widget::with_theme(
                    ConfirmationWindow::new(Callback::new(Rc::new(|widget, _| {
                        let parent = Widget::get_root(&widget);
                        let selector = Widget::downcast_kind_mut::<MainMenu>(&parent);
                        selector.next_step = Some(NextGameStep::Exit);
                    }))),
                    "exit_confirmation_window");
                exit_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&widget, exit_window);
            },
            _ => return false,
        }

        true
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to main menu widget");

        let title = Widget::with_theme(Label::empty(), "title");

        let module_title = Widget::with_theme(Label::empty(), "module_title");
        if Module::is_initialized() {
            module_title.borrow_mut().state.add_text_arg("module", &Module::campaign().name);
        }

        let menu_pane = Widget::empty("menu_pane");

        let new = Widget::with_theme(Button::empty(), "new");
        new.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let starter = Widget::downcast_kind_mut::<MainMenu>(&parent);

            parent.borrow_mut().invalidate_children();
            starter.mode = Mode::New;
            starter.content = Widget::with_defaults(CharacterSelector::new(parent.clone()));
        })));

        let load = Widget::with_theme(Button::empty(), "load");
        load.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let starter = Widget::downcast_kind_mut::<MainMenu>(&parent);

            starter.mode = Mode::Load;
            let load_window = LoadWindow::new(true);
            {
                let window = load_window.borrow();
                window.cancel.borrow_mut().state.set_visible(false);
            }
            starter.content = Widget::with_defaults(load_window);

            parent.borrow_mut().invalidate_children();
        })));

        let module = Widget::with_theme(Button::empty(), "module");
        module.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let window = Widget::downcast_kind_mut::<MainMenu>(&parent);

            window.mode = Mode::Module;
            let modules_list = Module::get_available_modules(
                &Config::resources_config().campaigns_directory);
            if modules_list.len() == 0 {
                util::error_and_exit("No valid modules found.");
            }
            let module_selector = ModuleSelector::new(modules_list);
            window.content = Widget::with_defaults(module_selector);

            parent.borrow_mut().invalidate_children();
        })));

        let options = Widget::with_theme(Button::empty(), "options");
        options.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let window = Widget::downcast_kind_mut::<MainMenu>(&parent);
            window.mode = Mode::Options;
            let configs = window.display_configurations.clone();

            window.content = Widget::with_defaults(Options::new(configs));

            parent.borrow_mut().invalidate_children();
        })));

        let links = Widget::with_theme(Button::empty(), "links");
        links.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let window = Widget::downcast_kind_mut::<MainMenu>(&parent);
            window.mode = Mode::Links;

            window.content = Widget::with_defaults(LinksPane::new());

            parent.borrow_mut().invalidate_children();
        })));

        let exit = Widget::with_theme(Button::empty(), "exit");
        exit.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(&widget, 2);
            let window = Widget::downcast_kind_mut::<MainMenu>(&parent);
            window.next_step = Some(NextGameStep::Exit);
        })));

        match self.mode {
            Mode::New => new.borrow_mut().state.set_active(true),
            Mode::Load => load.borrow_mut().state.set_active(true),
            Mode::Module => module.borrow_mut().state.set_active(true),
            Mode::Options => options.borrow_mut().state.set_active(true),
            Mode::Links => links.borrow_mut().state.set_active(true),
            Mode::NoChoice => (),
        }

        if !Module::is_initialized() {
            new.borrow_mut().state.set_enabled(false);
            load.borrow_mut().state.set_enabled(false);
            module_title.borrow_mut().state.set_visible(false);
        } else if !save_file::has_available_save_files() {
            load.borrow_mut().state.set_enabled(false);
        }

        Widget::add_children_to(&menu_pane, vec![module, new, load, options, links, exit]);

        let mut children = vec![title, module_title, menu_pane, self.content.clone()];

        if let Some(builder) = self.char_builder_to_add.take() {
            children.push(Widget::with_defaults(builder));
        }

        children
    }
}
