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

use std::any::Any;
use std::rc::Rc;
use std::cell::{RefCell};
use std::path::Path;

use sulis_core::config::{self, Config};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::io::{keyboard_event::Key, InputAction, DisplayConfiguration, event::ClickKind};
use sulis_core::config::DisplayMode;
use sulis_core::widgets::{Button, Label, ScrollPane};

use crate::main_menu::MainMenu;

enum Tab {
    Display,
    Input,
}

pub struct Options {
    display_confs: Vec<DisplayConfiguration>,

    cur_tab: Tab,

    cur_display_mode: DisplayMode,
    cur_display_conf: usize,
    cur_ui_scale: (i32, i32),
    cur_resolution: (u32, u32),
    cur_anim_speed: u32,
    cur_scroll_speed: f32,
    cur_edge_scrolling: bool,
    cur_keybindings: Vec<(Key, InputAction)>,
}

impl Options {
    pub fn new(display_confs: Vec<DisplayConfiguration>) -> Rc<RefCell<Options>> {
        let config = Config::get_clone();
        let mut cur_keybindings: Vec<_> =
            config.input.keybindings.iter().map(|(k, v)| (*k, *v)).collect();
        cur_keybindings.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

        let cur_display_conf = if config.display.monitor >= display_confs.len() {
            0
        } else {
            config.display.monitor
        };

        Rc::new(RefCell::new(Options {
            display_confs,

            cur_tab: Tab::Display,

            cur_display_mode: config.display.mode,
            cur_display_conf,
            cur_resolution: (config.display.width_pixels, config.display.height_pixels),
            cur_anim_speed: config.display.animation_base_time_millis,
            cur_scroll_speed: config.input.scroll_speed,
            cur_edge_scrolling: config.input.edge_scrolling,
            cur_ui_scale: (config.display.width, config.display.height),
            cur_keybindings,
        }))
    }

    fn reset_config(&self) {
        let path = Path::new(config::CONFIG_BASE);
        let config = match Config::new(&path) {
            Ok(config) => config,
            Err(e) => {
                warn!("Unable to get base config file at {}", config::CONFIG_BASE);
                warn!("{}", e);
                return;
            }
        };

        // don't save config to disk at this point.  it is only saved
        // when accepted in the save_or_revert_options_window popup
        Config::set(config);
    }

    fn save_current_config(&self) {
        let mut config = Config::get_clone();
        config.display.mode = self.cur_display_mode;
        config.display.animation_base_time_millis = self.cur_anim_speed;
        config.display.monitor = self.cur_display_conf;
        config.display.width_pixels = self.cur_resolution.0;
        config.display.height_pixels = self.cur_resolution.1;
        config.display.width = self.cur_ui_scale.0;
        config.display.height = self.cur_ui_scale.1;

        config.input.scroll_speed = self.cur_scroll_speed;
        config.input.edge_scrolling = self.cur_edge_scrolling;
        config.input.keybindings.clear();
        for (k, v) in self.cur_keybindings.iter() {
            config.input.keybindings.insert(*k, *v);
        }

        // don't save config to disk at this point.  it is only saved
        // when accepted in the save_or_revert_options_window popup
        Config::set(config);
    }

    fn add_display_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let mode_title = Widget::with_theme(Label::empty(), "mode_title");

        let mode_content = Widget::empty("mode_content");

        let mode_window = Widget::with_theme(Button::empty(), "mode_window");
        mode_window.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_display_mode = DisplayMode::Window;
            parent.borrow_mut().invalidate_children();
        })));
        let mode_borderless = Widget::with_theme(Button::empty(), "mode_borderless");
        mode_borderless.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_display_mode = DisplayMode::BorderlessWindow;
            parent.borrow_mut().invalidate_children();
        })));
        let mode_fullscreen = Widget::with_theme(Button::empty(), "mode_fullscreen");
        mode_fullscreen.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_display_mode = DisplayMode::Fullscreen;
            parent.borrow_mut().invalidate_children();
        })));

        match self.cur_display_mode {
            DisplayMode::Window => mode_window.borrow_mut().state.set_active(true),
            DisplayMode::BorderlessWindow => mode_borderless.borrow_mut().state.set_active(true),
            DisplayMode::Fullscreen => mode_fullscreen.borrow_mut().state.set_active(true),
        }

        Widget::add_child_to(&mode_content, mode_window);
        Widget::add_child_to(&mode_content, mode_borderless);
        Widget::add_child_to(&mode_content, mode_fullscreen);

        let monitor_title = Widget::with_theme(Label::empty(), "monitor_title");

        let monitor_content = Widget::empty("monitor_content");

        let monitor_label = Widget::with_theme(Label::empty(), "monitor_label");
        let name = self.display_confs[self.cur_display_conf].name.to_string();
        monitor_label.borrow_mut().state.add_text_arg("monitor", &name);
        let next_monitor = Widget::with_theme(Button::empty(), "next_monitor");
        next_monitor.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);

            if options.cur_display_conf == options.display_confs.len() - 1 {
                options.cur_display_conf = 0;
            } else {
                options.cur_display_conf += 1;
            }

            parent.borrow_mut().invalidate_children();
        })));
        if self.display_confs.len() == 1 {
            next_monitor.borrow_mut().state.set_enabled(false);
        }

        Widget::add_child_to(&monitor_content, monitor_label);
        Widget::add_child_to(&monitor_content, next_monitor);

        let resolution_title = Widget::with_theme(Label::empty(), "resolution_title");

        let scrollpane = ScrollPane::new();
        let resolution_pane = Widget::with_theme(scrollpane.clone(), "resolution_pane");
        let mut resolution_found = false;
        for (w, h) in self.display_confs[self.cur_display_conf].resolutions.iter() {
            let (w, h) = (*w, *h);
            let button = Widget::with_theme(Button::empty(), "resolution_button");
            button.borrow_mut().state.add_text_arg("width", &w.to_string());
            button.borrow_mut().state.add_text_arg("height", &h.to_string());

            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 4);
                let options = Widget::downcast_kind_mut::<Options>(&parent);
                options.cur_resolution = (w, h);
                parent.borrow_mut().invalidate_children();
            })));


            if w == self.cur_resolution.0 && h == self.cur_resolution.1 {
                button.borrow_mut().state.set_active(true);
                resolution_found = true;
            }

            scrollpane.borrow().add_to_content(button);
        }

        if !resolution_found {
            info!("Screen resolution is set to {},{} which is non-standard",
                  self.cur_resolution.0, self.cur_resolution.1);
        }

        let ui_scale_title = Widget::with_theme(Label::empty(), "ui_scale_title");

        let ui_scale_content = Widget::empty("ui_scale_content");

        let (ui_width, ui_height) = (self.cur_ui_scale.0, self.cur_ui_scale.1);
        let normal = Widget::with_theme(Button::empty(), "normal");
        normal.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_ui_scale = UI_SCALE_NORMAL;
            parent.borrow_mut().invalidate_children();
        })));

        let small = Widget::with_theme(Button::empty(), "small");
        small.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_ui_scale = UI_SCALE_SMALL;
            parent.borrow_mut().invalidate_children();
        })));

        if ui_width == UI_SCALE_NORMAL.0 && ui_height == UI_SCALE_NORMAL.1 {
            normal.borrow_mut().state.set_active(true);
        } else if ui_width == UI_SCALE_SMALL.0 && ui_height == UI_SCALE_SMALL.1 {
            small.borrow_mut().state.set_active(true);
        } else {
            info!("UI Scale is set to {},{} which is a nonstandard value", ui_width, ui_height);
        }

        Widget::add_child_to(&ui_scale_content, normal);
        Widget::add_child_to(&ui_scale_content, small);

        let anim_speed_title = Widget::with_theme(Label::empty(), "anim_speed_title");

        let anim_speed_content = Widget::empty("anim_speed_content");
        let mut speed_found = false;
        for speed in ANIM_SPEEDS.iter() {
            let speed = *speed;
            let button = Widget::with_theme(Button::empty(), "speed_button");
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 3);
                let options = Widget::downcast_kind_mut::<Options>(&parent);
                options.cur_anim_speed = speed;
                parent.borrow_mut().invalidate_children();
            })));
            if speed == self.cur_anim_speed {
                button.borrow_mut().state.set_active(true);
                speed_found = true;
            }

            Widget::add_child_to(&anim_speed_content, button);
        }

        if !speed_found {
            info!("Animation speed is set to {} which is a nonstandard value",
                  self.cur_anim_speed);
        }

        let slow_label = Widget::with_theme(Label::empty(), "anim_speed_slow");
        let fast_label = Widget::with_theme(Label::empty(), "anim_speed_fast");

        vec![mode_title, mode_content, monitor_title, monitor_content,
            resolution_title, resolution_pane, ui_scale_title, ui_scale_content,
            slow_label, fast_label, anim_speed_title, anim_speed_content]
    }

    fn add_input_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let scroll_speed_title = Widget::with_theme(Label::empty(), "scroll_speed_title");

        let scroll_speed_content = Widget::empty("scroll_speed_content");
        let mut speed_found = false;
        for speed in SCROLL_SPEEDS.iter() {
            let speed = *speed;
            let button = Widget::with_theme(Button::empty(), "speed_button");
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 3);
                let options = Widget::downcast_kind_mut::<Options>(&parent);
                options.cur_scroll_speed = speed;
                parent.borrow_mut().invalidate_children();
            })));
            if speed == self.cur_scroll_speed {
                button.borrow_mut().state.set_active(true);
                speed_found = true;
            }

            Widget::add_child_to(&scroll_speed_content, button);
        }

        if !speed_found {
            info!("Scroll speed is set to {} which is a nonstandard value",
                  self.cur_scroll_speed);
        }

        let slow_label = Widget::with_theme(Label::empty(), "scroll_speed_slow");
        let fast_label = Widget::with_theme(Label::empty(), "scroll_speed_fast");

        let edge_scroll_content = Widget::empty("edge_scroll_content");
        let edge_scroll_label = Widget::with_theme(Label::empty(), "label");
        Widget::add_child_to(&edge_scroll_content, edge_scroll_label);

        let edge_scroll_on = Widget::with_theme(Button::empty(), "on");
        edge_scroll_on.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_edge_scrolling = true;
            parent.borrow_mut().invalidate_children();
        })));
        let edge_scroll_off = Widget::with_theme(Button::empty(), "off");
        edge_scroll_off.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 3);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_edge_scrolling = false;
            parent.borrow_mut().invalidate_children();
        })));
        if self.cur_edge_scrolling {
            edge_scroll_on.borrow_mut().state.set_active(true);
        } else {
            edge_scroll_off.borrow_mut().state.set_active(true);
        }

        Widget::add_child_to(&edge_scroll_content, edge_scroll_on);
        Widget::add_child_to(&edge_scroll_content, edge_scroll_off);

        let keybindings_title = Widget::with_theme(Label::empty(), "keybindings_title");

        let scrollpane = ScrollPane::new();
        let keybindings_pane = Widget::with_theme(scrollpane.clone(), "keybindings_pane");
        for (key, action) in self.cur_keybindings.iter() {
            let key_button = Widget::with_theme(Button::empty(), "key_button");
            key_button.borrow_mut().state.add_text_arg("key", &format!("{:?}", key));

            let action_ref = action.clone();
            key_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 4);

                let root = Widget::get_root(widget);
                let popup = KeybindingPopup::new(action_ref.clone(), parent);
                Widget::add_child_to(&root, Widget::with_defaults(popup));
            })));

            let action_label= Widget::with_theme(Label::empty(), "action_label");
            action_label.borrow_mut().state.add_text_arg("action", &format!("{:?}", action));

            scrollpane.borrow().add_to_content(key_button);
            scrollpane.borrow().add_to_content(action_label);
        }

        vec![scroll_speed_title, scroll_speed_content, fast_label,
            slow_label, keybindings_title, keybindings_pane, edge_scroll_content]
    }
}

const UI_SCALE_NORMAL: (i32, i32) = (320, 180);
const UI_SCALE_SMALL: (i32, i32) = (368, 207);
const ANIM_SPEEDS: [u32; 5] = [75, 50, 35, 25, 15];
const SCROLL_SPEEDS: [f32; 7] = [0.75, 1.0, 1.5, 2.25, 3.5, 5.0, 7.0];

impl WidgetKind for Options {
    widget_kind!("options_window");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let apply = Widget::with_theme(Button::empty(), "apply");
        apply.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::go_up_tree(widget, 1);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.save_current_config();

            let root = Widget::get_root(&widget);
            let menu = Widget::downcast_kind_mut::<MainMenu>(&root);
            menu.recreate_io();
        })));

        let reset = Widget::with_theme(Button::empty(), "reset");
        reset.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.reset_config();

            let root = Widget::get_root(&widget);
            let menu = Widget::downcast_kind_mut::<MainMenu>(&root);
            menu.recreate_io();
        })));

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let root = Widget::get_root(&widget);
            let menu = Widget::downcast_kind_mut::<MainMenu>(&root);
            menu.reset();
            root.borrow_mut().invalidate_children();
        })));

        let display = Widget::with_theme(Button::empty(), "display");
        display.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_tab = Tab::Display;
            parent.borrow_mut().invalidate_children();
        })));

        let input = Widget::with_theme(Button::empty(), "input");
        input.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let options = Widget::downcast_kind_mut::<Options>(&parent);
            options.cur_tab = Tab::Input;
            parent.borrow_mut().invalidate_children();
        })));

        let content = Widget::empty("content");

        let widgets = match self.cur_tab {
            Tab::Display => {
                display.borrow_mut().state.set_active(true);
                self.add_display_widgets()
            },
            Tab::Input => {
                input.borrow_mut().state.set_active(true);
                self.add_input_widgets()
            }
        };

        Widget::add_children_to(&content, widgets);

        vec![title, apply, cancel, reset, content, display, input]
    }
}

pub struct KeybindingPopup {
    action: InputAction,
    options_widget: Rc<RefCell<Widget>>,
}

impl KeybindingPopup {
    pub fn new(action: InputAction,
               options_widget: Rc<RefCell<Widget>>) -> Rc<RefCell<KeybindingPopup>> {
        Rc::new(RefCell::new(KeybindingPopup { action, options_widget }))
    }
}

impl WidgetKind for KeybindingPopup {
    widget_kind!("keybinding_popup");

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        widget.borrow_mut().mark_for_removal();
        true
    }

    fn on_raw_key(&mut self, widget: &Rc<RefCell<Widget>>, key: Key) -> bool {
        let options = Widget::downcast_kind_mut::<Options>(&self.options_widget);
        let mut matched_index = 0;
        for (index, (_, v)) in options.cur_keybindings.iter().enumerate() {
            if *v == self.action {
                matched_index = index;
                break;
            }
        }

        options.cur_keybindings[matched_index] = (key, self.action.clone());
        self.options_widget.borrow_mut().invalidate_children();
        widget.borrow_mut().mark_for_removal();
        false
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("action", &format!("{:?}", self.action));

        vec![title]
    }
}

