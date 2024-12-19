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
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use sulis_core::config::DisplayMode;
use sulis_core::config::{self, Config, RawClick};
use sulis_core::io::{
    event::ClickKind, keyboard_event::Key, DisplayConfiguration, InputActionKind,
};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label, ScrollDirection, ScrollPane, TextArea};

use crate::main_menu::MainMenu;

enum Tab {
    Display,
    Input,
    Gameplay,
    Audio,
}

pub struct Options {
    display_confs: Vec<DisplayConfiguration>,

    cur_tab: Tab,

    cur_display_mode: DisplayMode,
    cur_vsync: bool,
    cur_display_conf: usize,
    cur_ui_scale: (i32, i32),
    cur_resolution: (u32, u32),
    cur_default_zoom: f32,
    cur_anim_speed: u32,
    cur_scroll_speed: f32,
    cur_edge_scrolling: bool,
    cur_keybindings: Vec<(Key, InputActionKind)>,
    cur_click_actions: Vec<(RawClick, ClickKind)>,

    cur_crit_screen_shake: bool,
    cur_scroll_to_active: bool,

    audio_devices: Vec<String>,
    cur_audio_device: Option<usize>,
    master_volume: f32,
    music_volume: f32,
    effects_volume: f32,
    ambient_volume: f32,
}

impl Options {
    pub fn new(
        display_confs: Vec<DisplayConfiguration>,
        audio_devices: Vec<String>,
    ) -> Rc<RefCell<Options>> {
        let config = Config::get_clone();
        let mut cur_keybindings: Vec<_> = config
            .input
            .keybindings
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        cur_keybindings.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

        let mut cur_click_actions: Vec<_> = config
            .input
            .click_actions
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        cur_click_actions.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

        let cur_display_conf = if config.display.monitor >= display_confs.len() {
            0
        } else {
            config.display.monitor
        };

        let cur_audio_device = if audio_devices.is_empty() {
            None
        } else if config.audio.device < audio_devices.len() {
            Some(config.audio.device)
        } else {
            Some(0)
        };

        Rc::new(RefCell::new(Options {
            display_confs,

            cur_tab: Tab::Display,

            cur_display_mode: config.display.mode,
            cur_vsync: config.display.vsync_enabled,
            cur_display_conf,
            cur_default_zoom: config.display.default_zoom,
            cur_resolution: (config.display.width_pixels, config.display.height_pixels),
            cur_anim_speed: config.display.animation_base_time_millis,
            cur_scroll_speed: config.input.scroll_speed,
            cur_edge_scrolling: config.input.edge_scrolling,
            cur_ui_scale: (config.display.width, config.display.height),
            cur_keybindings,
            cur_click_actions,

            cur_crit_screen_shake: config.input.crit_screen_shake,
            cur_scroll_to_active: config.display.scroll_to_active,

            audio_devices,
            cur_audio_device,
            master_volume: config.audio.master_volume,
            music_volume: config.audio.music_volume,
            effects_volume: config.audio.effects_volume,
            ambient_volume: config.audio.ambient_volume,
        }))
    }

    fn reset_config(&self) {
        let path = Path::new(config::CONFIG_BASE);
        let config = match Config::new(path, 0) {
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
        config.display.vsync_enabled = self.cur_vsync;
        config.display.animation_base_time_millis = self.cur_anim_speed;
        config.display.monitor = self.cur_display_conf;
        config.display.width_pixels = self.cur_resolution.0;
        config.display.height_pixels = self.cur_resolution.1;
        config.display.width = self.cur_ui_scale.0;
        config.display.height = self.cur_ui_scale.1;
        config.display.default_zoom = self.cur_default_zoom;

        config.input.scroll_speed = self.cur_scroll_speed;
        config.input.edge_scrolling = self.cur_edge_scrolling;
        config.input.click_actions.clear();
        for (k, v) in self.cur_click_actions.iter() {
            config.input.click_actions.insert(*k, *v);
        }

        config.input.keybindings.clear();
        for (k, v) in self.cur_keybindings.iter() {
            config.input.keybindings.insert(*k, *v);
        }

        config.input.crit_screen_shake = self.cur_crit_screen_shake;
        config.display.scroll_to_active = self.cur_scroll_to_active;

        config.audio.device = self.cur_audio_device.unwrap_or(0);
        config.audio.master_volume = self.master_volume;
        config.audio.music_volume = self.music_volume;
        config.audio.effects_volume = self.effects_volume;
        config.audio.ambient_volume = self.ambient_volume;

        // don't save config to disk at this point.  it is only saved
        // when accepted in the save_or_revert_options_window popup
        Config::set(config);
    }

    fn add_display_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let mode_title = Widget::with_theme(Label::empty(), "mode_title");

        let mode_content = Widget::empty("mode_content");

        let mode_window = Widget::with_theme(Button::empty(), "mode_window");
        mode_window
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_display_mode = DisplayMode::Window;
                parent.borrow_mut().invalidate_children();
            })));
        let mode_borderless = Widget::with_theme(Button::empty(), "mode_borderless");
        mode_borderless
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_display_mode = DisplayMode::BorderlessWindow;
                parent.borrow_mut().invalidate_children();

                match options.display_confs[options.cur_display_conf]
                    .resolutions
                    .first()
                {
                    None => {
                        warn!("No valid resolutions for current display mode.");
                    }
                    Some(res) => {
                        // set the resolution to the first entry which is the largest
                        options.cur_resolution = (res.width, res.height);
                    }
                };
            })));
        let mode_fullscreen = Widget::with_theme(Button::empty(), "mode_fullscreen");
        mode_fullscreen
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
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

        let vsync_on = Widget::with_theme(Button::empty(), "on");
        vsync_on
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_vsync = true;
                parent.borrow_mut().invalidate_children();
            })));
        let vsync_off = Widget::with_theme(Button::empty(), "off");
        vsync_off
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_vsync = false;
                parent.borrow_mut().invalidate_children();
            })));
        if self.cur_vsync {
            vsync_on.borrow_mut().state.set_active(true);
        } else {
            vsync_off.borrow_mut().state.set_active(true);
        }

        let vsync_content = Widget::empty("vsync_content");
        let vsync_label = Widget::with_theme(Label::empty(), "label");
        Widget::add_child_to(&vsync_content, vsync_label);
        Widget::add_child_to(&vsync_content, vsync_on);
        Widget::add_child_to(&vsync_content, vsync_off);

        let monitor_title = Widget::with_theme(Label::empty(), "monitor_title");

        let monitor_content = Widget::empty("monitor_content");

        let monitor_label = Widget::with_theme(Label::empty(), "monitor_label");
        let name = self.display_confs[self.cur_display_conf].name.to_string();
        monitor_label
            .borrow_mut()
            .state
            .add_text_arg("monitor", &name);
        let next_monitor = Widget::with_theme(Button::empty(), "next_monitor");
        next_monitor
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);

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

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        let resolution_pane = Widget::with_theme(scrollpane.clone(), "resolution_pane");
        let mut resolution_found = false;
        for res in self.display_confs[self.cur_display_conf].resolutions.iter() {
            let button = Widget::with_theme(Button::empty(), "resolution_button");

            {
                let state = &mut button.borrow_mut().state;

                state.add_text_arg("width", &res.width.to_string());
                state.add_text_arg("height", &res.height.to_string());
                let (w, h) = (res.width, res.height);
                state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, options) = Widget::parent_mut::<Options>(widget);
                    options.cur_resolution = (w, h);
                    parent.borrow_mut().invalidate_children();
                })));

                if res.width == self.cur_resolution.0 && res.height == self.cur_resolution.1 {
                    resolution_found = true;
                    state.set_active(true);
                }

                match self.cur_display_mode {
                    DisplayMode::Window => (),
                    DisplayMode::BorderlessWindow => {
                        if !res.monitor_size {
                            continue;
                        }
                        state.set_enabled(false);
                    }
                    DisplayMode::Fullscreen => {
                        if !res.fullscreen {
                            continue;
                        }
                    }
                }
            }

            scrollpane.borrow().add_to_content(button);
        }

        if !resolution_found {
            info!(
                "Screen resolution is set to {},{} which is non-standard",
                self.cur_resolution.0, self.cur_resolution.1
            );
        }

        let ui_scale_title = Widget::with_theme(Label::empty(), "ui_scale_title");

        let ui_scale_content = Widget::empty("ui_scale_content");

        let (ui_width, ui_height) = (self.cur_ui_scale.0, self.cur_ui_scale.1);
        let normal = Widget::with_theme(Button::empty(), "normal");
        normal
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_ui_scale = UI_SCALE_NORMAL;
                parent.borrow_mut().invalidate_children();
            })));

        let small = Widget::with_theme(Button::empty(), "small");
        small
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_ui_scale = UI_SCALE_SMALL;
                parent.borrow_mut().invalidate_children();
            })));

        if ui_width == UI_SCALE_NORMAL.0 && ui_height == UI_SCALE_NORMAL.1 {
            normal.borrow_mut().state.set_active(true);
        } else if ui_width == UI_SCALE_SMALL.0 && ui_height == UI_SCALE_SMALL.1 {
            small.borrow_mut().state.set_active(true);
        } else {
            info!(
                "UI Scale is set to {},{} which is a nonstandard value",
                ui_width, ui_height
            );
        }

        Widget::add_child_to(&ui_scale_content, normal);
        Widget::add_child_to(&ui_scale_content, small);

        vec![
            mode_title,
            mode_content,
            vsync_content,
            monitor_title,
            monitor_content,
            resolution_title,
            resolution_pane,
            ui_scale_title,
            ui_scale_content,
        ]
    }

    fn add_audio_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let device_content = Widget::empty("device_content");

        let device_title = Widget::with_theme(Label::empty(), "device_title");

        let device_label = Widget::with_theme(Label::empty(), "device_label");
        let name = match self.cur_audio_device {
            None => "No Audio Device Detected",
            Some(idx) => &self.audio_devices[idx],
        };
        device_label.borrow_mut().state.add_text_arg("device", name);
        let next_device = Widget::with_theme(Button::empty(), "next_device");
        next_device
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);

                let cur_value = options.cur_audio_device.unwrap_or(0);

                if cur_value == options.audio_devices.len() - 1 {
                    options.cur_audio_device = Some(0);
                } else {
                    options.cur_audio_device = Some(cur_value + 1);
                }

                parent.borrow_mut().invalidate_children();
            })));
        if self.audio_devices.len() < 2 {
            next_device.borrow_mut().state.set_enabled(false);
        }
        Widget::add_child_to(&device_content, device_label);
        Widget::add_child_to(&device_content, next_device);

        vec![
            device_content,
            device_title,
            self.add_volume_widget("master", self.master_volume, |o, vol| o.master_volume = vol),
            self.add_volume_widget("music", self.music_volume, |o, vol| o.music_volume = vol),
            self.add_volume_widget("effects", self.effects_volume, |o, vol| {
                o.effects_volume = vol
            }),
            self.add_volume_widget("ambient", self.ambient_volume, |o, vol| {
                o.ambient_volume = vol
            }),
        ]
    }

    fn add_volume_widget(
        &self,
        id: &str,
        cur: f32,
        setter: fn(&mut Options, f32),
    ) -> Rc<RefCell<Widget>> {
        let content = Widget::empty(&format!("{id}_volume_content"));

        let title = Widget::with_theme(Label::empty(), "title");
        Widget::add_child_to(&content, title);

        let buttons = Widget::empty("buttons");
        let mut found = false;
        for vol in VOLUME_LEVELS.iter() {
            let vol = *vol;
            let button = Widget::with_theme(Button::empty(), "volume_button");
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, options) = Widget::parent_mut::<Options>(widget);
                    setter(options, vol);
                    parent.borrow_mut().invalidate_children();
                })));
            if (vol - cur).abs() < f32::EPSILON {
                button.borrow_mut().state.set_active(true);
                found = true;
            }

            Widget::add_child_to(&buttons, button);
        }
        Widget::add_child_to(&content, buttons);

        let soft_label = Widget::with_theme(Label::empty(), "soft");
        Widget::add_child_to(&content, soft_label);
        let loud_label = Widget::with_theme(Label::empty(), "loud");
        Widget::add_child_to(&content, loud_label);

        if !found {
            info!("Volume set to nonstandard value: {}", cur);
        }

        content
    }

    fn add_gameplay_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let screen_shake_on = Widget::with_theme(Button::empty(), "on");
        screen_shake_on
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_crit_screen_shake = true;
                parent.borrow_mut().invalidate_children();
            })));
        let screen_shake_off = Widget::with_theme(Button::empty(), "off");
        screen_shake_off
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_crit_screen_shake = false;
                parent.borrow_mut().invalidate_children();
            })));
        if self.cur_crit_screen_shake {
            screen_shake_on.borrow_mut().state.set_active(true);
        } else {
            screen_shake_off.borrow_mut().state.set_active(true);
        }

        let screen_shake_content = Widget::empty("screen_shake_content");
        Widget::add_child_to(&screen_shake_content, screen_shake_on);
        Widget::add_child_to(&screen_shake_content, screen_shake_off);

        let scroll_to_active_on = Widget::with_theme(Button::empty(), "on");
        scroll_to_active_on
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_scroll_to_active = true;
                parent.borrow_mut().invalidate_children();
            })));

        let scroll_to_active_off = Widget::with_theme(Button::empty(), "off");
        scroll_to_active_off
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_scroll_to_active = false;
                parent.borrow_mut().invalidate_children();
            })));
        if self.cur_scroll_to_active {
            scroll_to_active_on.borrow_mut().state.set_active(true);
        } else {
            scroll_to_active_off.borrow_mut().state.set_active(true);
        }

        let scroll_to_active_content = Widget::empty("scroll_to_active_content");
        Widget::add_child_to(&scroll_to_active_content, scroll_to_active_on);
        Widget::add_child_to(&scroll_to_active_content, scroll_to_active_off);

        let zoom_content = Widget::empty("default_zoom_content");
        let mut zoom_found = false;
        for zoom in DEFAULT_ZOOMS.iter() {
            let zoom = *zoom;
            let button = Widget::with_theme(Button::empty(), "default_zoom_button");
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, options) = Widget::parent_mut::<Options>(widget);
                    options.cur_default_zoom = zoom;
                    parent.borrow_mut().invalidate_children();
                })));
            if (zoom - self.cur_default_zoom).abs() < f32::EPSILON {
                button.borrow_mut().state.set_active(true);
                zoom_found = true;
            }
            Widget::add_child_to(&zoom_content, button);
        }

        if !zoom_found {
            info!(
                "Default zoom set to non standard value {}",
                self.cur_default_zoom
            );
        }

        let anim_speed_title = Widget::with_theme(Label::empty(), "anim_speed_title");

        let anim_speed_content = Widget::empty("anim_speed_content");
        let mut speed_found = false;
        for speed in ANIM_SPEEDS.iter() {
            let speed = *speed;
            let button = Widget::with_theme(Button::empty(), "speed_button");
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, options) = Widget::parent_mut::<Options>(widget);
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
            info!(
                "Animation speed is set to {} which is a nonstandard value",
                self.cur_anim_speed
            );
        }

        let slow_label = Widget::with_theme(Label::empty(), "anim_speed_slow");
        let fast_label = Widget::with_theme(Label::empty(), "anim_speed_fast");

        vec![
            screen_shake_content,
            slow_label,
            fast_label,
            anim_speed_title,
            anim_speed_content,
            zoom_content,
            scroll_to_active_content,
        ]
    }

    fn add_input_widgets(&mut self) -> Vec<Rc<RefCell<Widget>>> {
        let scroll_speed_title = Widget::with_theme(Label::empty(), "scroll_speed_title");

        let scroll_speed_content = Widget::empty("scroll_speed_content");
        let mut speed_found = false;
        for speed in SCROLL_SPEEDS.iter() {
            let speed = *speed;
            let button = Widget::with_theme(Button::empty(), "speed_button");
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, options) = Widget::parent_mut::<Options>(widget);
                    options.cur_scroll_speed = speed;
                    parent.borrow_mut().invalidate_children();
                })));
            if (speed - self.cur_scroll_speed).abs() < f32::EPSILON {
                button.borrow_mut().state.set_active(true);
                speed_found = true;
            }

            Widget::add_child_to(&scroll_speed_content, button);
        }

        if !speed_found {
            info!(
                "Scroll speed is set to {} which is a nonstandard value",
                self.cur_scroll_speed
            );
        }

        let slow_label = Widget::with_theme(Label::empty(), "scroll_speed_slow");
        let fast_label = Widget::with_theme(Label::empty(), "scroll_speed_fast");

        let edge_scroll_content = Widget::empty("edge_scroll_content");
        let edge_scroll_label = Widget::with_theme(Label::empty(), "label");
        Widget::add_child_to(&edge_scroll_content, edge_scroll_label);

        let edge_scroll_on = Widget::with_theme(Button::empty(), "on");
        edge_scroll_on
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_edge_scrolling = true;
                parent.borrow_mut().invalidate_children();
            })));
        let edge_scroll_off = Widget::with_theme(Button::empty(), "off");
        edge_scroll_off
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
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

        let mouse_title = Widget::with_theme(Label::empty(), "mouse_title");
        let mouse_pane = Widget::empty("mouse_pane");
        for (raw, action) in &self.cur_click_actions {
            let action = *action;
            let row = Widget::empty("row");

            let button = Widget::with_theme(Button::empty(), "mouse_button");
            button
                .borrow_mut()
                .state
                .add_text_arg("button", &format!("{raw:?}"));
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, _) = Widget::parent::<Options>(widget);

                    let root = Widget::get_root(widget);
                    let popup = MousePopup::new(action, parent);
                    Widget::add_child_to(&root, Widget::with_defaults(popup));
                })));

            let label = Widget::with_theme(TextArea::empty(), "action_label");
            label
                .borrow_mut()
                .state
                .add_text_arg(&format!("{action:?}"), "true");

            Widget::add_children_to(&row, vec![button, label]);

            Widget::add_child_to(&mouse_pane, row);
        }

        let keybindings_title = Widget::with_theme(Label::empty(), "keybindings_title");

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        let keybindings_pane = Widget::with_theme(scrollpane.clone(), "keybindings_pane");
        for (key, action) in self.cur_keybindings.iter() {
            let key_button = Widget::with_theme(Button::empty(), "key_button");
            key_button
                .borrow_mut()
                .state
                .add_text_arg("key", &format!("{key:?}"));

            let action_ref = *action;
            key_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, _) = Widget::parent::<Options>(widget);

                    let root = Widget::get_root(widget);
                    let popup = KeybindingPopup::new(action_ref, parent);
                    Widget::add_child_to(&root, Widget::with_defaults(popup));
                })));

            let action_label = Widget::with_theme(Label::empty(), "action_label");
            action_label
                .borrow_mut()
                .state
                .add_text_arg("action", &format!("{action:?}"));

            scrollpane.borrow().add_to_content(key_button);
            scrollpane.borrow().add_to_content(action_label);
        }

        vec![
            scroll_speed_title,
            scroll_speed_content,
            fast_label,
            slow_label,
            mouse_title,
            mouse_pane,
            keybindings_title,
            keybindings_pane,
            edge_scroll_content,
        ]
    }
}

const VOLUME_LEVELS: [f32; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

const UI_SCALE_NORMAL: (i32, i32) = (320, 180);
const UI_SCALE_SMALL: (i32, i32) = (368, 207);
const ANIM_SPEEDS: [u32; 5] = [75, 50, 35, 25, 15];
const DEFAULT_ZOOMS: [f32; 5] = [1.0, 1.2, 1.4, 1.6, 1.8];
const SCROLL_SPEEDS: [f32; 7] = [0.75, 1.0, 1.5, 2.25, 3.5, 5.0, 7.0];

impl WidgetKind for Options {
    widget_kind!("options_window");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let apply = Widget::with_theme(Button::empty(), "apply");
        apply
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.save_current_config();

                let (_, menu) = Widget::parent_mut::<MainMenu>(&parent);
                menu.recreate_io();
            })));

        let reset = Widget::with_theme(Button::empty(), "reset");
        reset
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.reset_config();

                let (_, menu) = Widget::parent_mut::<MainMenu>(&parent);
                menu.recreate_io();
            })));

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (root, menu) = Widget::parent_mut::<MainMenu>(widget);
                menu.reset();
                root.borrow_mut().invalidate_children();
            })));

        let display = Widget::with_theme(Button::empty(), "display");
        display
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_tab = Tab::Display;
                parent.borrow_mut().invalidate_children();
            })));

        let input = Widget::with_theme(Button::empty(), "input");
        input
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_tab = Tab::Input;
                parent.borrow_mut().invalidate_children();
            })));

        let gameplay = Widget::with_theme(Button::empty(), "gameplay");
        gameplay
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_tab = Tab::Gameplay;
                parent.borrow_mut().invalidate_children();
            })));

        let audio = Widget::with_theme(Button::empty(), "audio");
        audio
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, options) = Widget::parent_mut::<Options>(widget);
                options.cur_tab = Tab::Audio;
                parent.borrow_mut().invalidate_children();
            })));

        let content = Widget::empty("content");

        let widgets = match self.cur_tab {
            Tab::Display => {
                display.borrow_mut().state.set_active(true);
                self.add_display_widgets()
            }
            Tab::Input => {
                input.borrow_mut().state.set_active(true);
                self.add_input_widgets()
            }
            Tab::Gameplay => {
                gameplay.borrow_mut().state.set_active(true);
                self.add_gameplay_widgets()
            }
            Tab::Audio => {
                audio.borrow_mut().state.set_active(true);
                self.add_audio_widgets()
            }
        };

        Widget::add_children_to(&content, widgets);

        vec![
            title, apply, cancel, reset, content, display, input, gameplay, audio,
        ]
    }
}

pub struct MousePopup {
    action: ClickKind,
    options_widget: Rc<RefCell<Widget>>,
}

impl MousePopup {
    pub fn new(action: ClickKind, options_widget: Rc<RefCell<Widget>>) -> Rc<RefCell<MousePopup>> {
        Rc::new(RefCell::new(MousePopup {
            action,
            options_widget,
        }))
    }
}

impl WidgetKind for MousePopup {
    widget_kind!("mouse_popup");

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);

        let options = Widget::kind_mut::<Options>(&self.options_widget);
        let mut actions: HashMap<_, _> = options
            .cur_click_actions
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect();

        let old_click = *actions.get(&kind).unwrap();
        let new_click = *actions.get(&self.action).unwrap();

        actions.insert(kind, new_click);
        actions.insert(self.action, old_click);

        options.cur_click_actions = actions.into_iter().map(|(k, v)| (v, k)).collect();
        options
            .cur_click_actions
            .sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

        self.options_widget.borrow_mut().invalidate_children();
        widget.borrow_mut().mark_for_removal();
        true
    }

    fn on_raw_key(&mut self, widget: &Rc<RefCell<Widget>>, _: Key) -> bool {
        widget.borrow_mut().mark_for_removal();
        true
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");
        title
            .borrow_mut()
            .state
            .add_text_arg("action", &format!("{:?}", self.action));

        vec![title]
    }
}

pub struct KeybindingPopup {
    action: InputActionKind,
    options_widget: Rc<RefCell<Widget>>,
}

impl KeybindingPopup {
    pub fn new(
        action: InputActionKind,
        options_widget: Rc<RefCell<Widget>>,
    ) -> Rc<RefCell<KeybindingPopup>> {
        Rc::new(RefCell::new(KeybindingPopup {
            action,
            options_widget,
        }))
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
        let options = Widget::kind_mut::<Options>(&self.options_widget);
        let mut matched_index = 0;
        for (index, (_, v)) in options.cur_keybindings.iter().enumerate() {
            if *v == self.action {
                matched_index = index;
                break;
            }
        }

        options.cur_keybindings[matched_index] = (key, self.action);
        self.options_widget.borrow_mut().invalidate_children();
        widget.borrow_mut().mark_for_removal();
        false
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");
        title
            .borrow_mut()
            .state
            .add_text_arg("action", &format!("{:?}", self.action));

        vec![title]
    }
}
