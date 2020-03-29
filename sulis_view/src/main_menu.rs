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

mod mods_selector;
use self::mods_selector::ModsSelector;

pub mod options;
pub use self::options::Options;

pub mod save_or_revert_options_window;
pub use self::save_or_revert_options_window::SaveOrRevertOptionsWindow;

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::config::Config;
use sulis_core::io::{DisplayConfiguration, InputAction, MainLoopUpdater, AudioDeviceInfo};
use sulis_core::ui::*;
use sulis_core::util;
use sulis_core::widgets::{Button, ConfirmationWindow, TextArea};
use sulis_module::{modification, Module};
use sulis_state::{save_file, NextGameStep};

use crate::{CharacterBuilder, LoadWindow};

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
    fn update(&self, _root: &Rc<RefCell<Widget>>, _millis: u32) {}

    fn is_exit(&self) -> bool {
        self.view.borrow().is_exit()
    }
}

enum Mode {
    New,
    Load,
    Module,
    Mods,
    Options,
    Links,
    NoChoice,
}

pub struct MainMenu {
    pub(crate) next_step: Option<NextGameStep>,
    mode: Mode,
    content: Rc<RefCell<Widget>>,
    pub(crate) char_builder_to_add: Option<Rc<RefCell<CharacterBuilder>>>,
    pub(crate) display_configurations: Vec<DisplayConfiguration>,
    pub(crate) audio_devices: Vec<AudioDeviceInfo>,

    tip_text: Option<String>,
}

impl MainMenu {
    pub fn new(
        display_configurations: Vec<DisplayConfiguration>,
        audio_devices: Vec<AudioDeviceInfo>,
    ) -> Rc<RefCell<MainMenu>> {
        let tip_text = if Module::is_initialized() {
            Some(Module::rules().random_hint())
        } else {
            None
        };

        Rc::new(RefCell::new(MainMenu {
            next_step: None,
            mode: Mode::NoChoice,
            content: Widget::empty("content"),
            char_builder_to_add: None,
            display_configurations,
            audio_devices,
            tip_text,
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
}

impl WidgetKind for MainMenu {
    widget_kind!("main_menu");

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            Exit | Back => {
                let exit_window = Widget::with_theme(
                    ConfirmationWindow::new(Callback::new(Rc::new(|widget, _| {
                        let (_, selector) = Widget::parent_mut::<MainMenu>(widget);
                        selector.next_step = Some(NextGameStep::Exit);
                    }))),
                    "exit_confirmation_window",
                );
                exit_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&widget, exit_window);
            }
            _ => return false,
        }

        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        let theme = &widget.theme;
        if let Some(music_id) = theme.custom.get("default_music") {
            if !Module::is_initialized() {
                sulis_core::io::Audio::play_music(music_id, 1.0);
            } else {
                Module::rules().play_main_menu_music();
            }
            sulis_core::io::Audio::stop_ambient();
        }
        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to main menu widget");

        let title = Widget::with_theme(TextArea::empty(), "title");

        let background = Widget::empty("background");

        let module_title = Widget::with_theme(TextArea::empty(), "module_title");
        if Module::is_initialized() {
            let campaign = Module::campaign();
            module_title
                .borrow_mut()
                .state
                .add_text_arg("module", &campaign.name);
            if let Some(group) = &campaign.group {
                module_title
                    .borrow_mut()
                    .state
                    .add_text_arg("group", &group.name);
            }
        }

        let menu_pane = Widget::empty("menu_pane");

        let new = Widget::with_theme(Button::empty(), "new");
        new.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, starter) = Widget::parent_mut::<MainMenu>(widget);

                parent.borrow_mut().invalidate_children();
                starter.mode = Mode::New;
                starter.content = Widget::with_defaults(CharacterSelector::new(parent));
            })));

        let load = Widget::with_theme(Button::empty(), "load");
        load.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, starter) = Widget::parent_mut::<MainMenu>(widget);

                starter.mode = Mode::Load;
                let load_window = LoadWindow::new(true);
                {
                    let window = load_window.borrow();
                    window.cancel.borrow_mut().state.set_visible(false);
                }
                starter.content = Widget::with_defaults(load_window);

                parent.borrow_mut().invalidate_children();
            })));

        let mods = Widget::with_theme(Button::empty(), "mods");
        mods.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, view) = Widget::parent_mut::<MainMenu>(widget);

                let mods_list = modification::get_available_modifications();

                parent.borrow_mut().invalidate_children();
                view.mode = Mode::Mods;
                view.content = Widget::with_defaults(ModsSelector::new(mods_list));
            })));

        let module = Widget::with_theme(Button::empty(), "module");
        module
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<MainMenu>(widget);

                window.mode = Mode::Module;
                let modules_list = Module::get_available_modules();
                if modules_list.is_empty() {
                    util::error_and_exit("No valid modules found.");
                }
                let module_selector = ModuleSelector::new(modules_list);
                window.content = Widget::with_defaults(module_selector);

                parent.borrow_mut().invalidate_children();
            })));

        let options = Widget::with_theme(Button::empty(), "options");
        options
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<MainMenu>(widget);
                window.mode = Mode::Options;
                let configs = window.display_configurations.clone();
                let audio = window.audio_devices.clone();

                window.content = Widget::with_defaults(Options::new(configs, audio));

                parent.borrow_mut().invalidate_children();
            })));

        let links = Widget::with_theme(Button::empty(), "links");
        links
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<MainMenu>(widget);
                window.mode = Mode::Links;

                window.content = Widget::with_defaults(LinksPane::new());

                parent.borrow_mut().invalidate_children();
            })));

        let exit = Widget::with_theme(Button::empty(), "exit");
        exit.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (_, window) = Widget::parent_mut::<MainMenu>(widget);
                window.next_step = Some(NextGameStep::Exit);
            })));

        match self.mode {
            Mode::New => new.borrow_mut().state.set_active(true),
            Mode::Load => load.borrow_mut().state.set_active(true),
            Mode::Mods => mods.borrow_mut().state.set_active(true),
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

        Widget::add_children_to(
            &menu_pane,
            vec![module, new, load, mods, options, links, exit],
        );

        let mut children = vec![background, title, module_title, menu_pane];

        if let Some(tip_text) = &self.tip_text {
            let tip_pane = Widget::empty("tip_pane");
            let tip = Widget::with_theme(TextArea::empty(), "tip");
            tip.borrow_mut().state.add_text_arg("tip", tip_text);
            Widget::add_child_to(&tip_pane, tip);
            children.push(tip_pane);
        }

        children.push(self.content.clone());

        if let Some(builder) = self.char_builder_to_add.take() {
            children.push(Widget::with_defaults(builder));
        }

        if let Some(options) = Config::take_old_config() {
            let config_confirm = Widget::with_defaults(SaveOrRevertOptionsWindow::new(options));
            config_confirm.borrow_mut().state.set_modal(true);
            children.push(config_confirm);
        }

        children
    }
}
