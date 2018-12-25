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
use std::cell::RefCell;

use crate::{Button, Label, main_menu::MainMenu};
use sulis_core::config::{self, Config};
use sulis_core::resource::write_to_file;
use sulis_core::ui::{Callback, Widget, WidgetKind};

pub struct SaveOrRevertOptionsWindow {
    old_config: Config,
    elapsed: u32,
    timer: Rc<RefCell<Widget>>,
}

const TIMER_LEN: u32 = 10000;

impl SaveOrRevertOptionsWindow {
    pub fn new(old_config: Config) -> Rc<RefCell<SaveOrRevertOptionsWindow>> {
        let timer = Widget::with_theme(Label::empty(), "timer");
        timer.borrow_mut().state.text = (TIMER_LEN / 1000).to_string();
        Rc::new(RefCell::new(SaveOrRevertOptionsWindow {
            old_config,
            elapsed: 0,
            timer,
        }))
    }

    fn revert(&self, widget: &Rc<RefCell<Widget>>) {
        let config = self.old_config.clone();

        Config::set(config);
        Config::take_old_config(); // throw away reverted config

        let root = Widget::get_root(widget);
        let main_menu = Widget::downcast_kind_mut::<MainMenu>(&root);
        main_menu.recreate_io();
    }

    fn accept(&self, widget: &Rc<RefCell<Widget>>) {
        let config = Config::get_clone();
        let mut path = config::USER_DIR.clone();
        path.push("config.yml");
        if let Err(e) = write_to_file(path.as_path(), &config) {
            warn!("Error writing config to file: {:?}", path);
            warn!("{}", e);
        }

        widget.borrow_mut().mark_for_removal();
    }
}

impl WidgetKind for SaveOrRevertOptionsWindow {
    widget_kind!("save_or_revert_options_window");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn update(&mut self, widget: &Rc<RefCell<Widget>>, millis: u32) {
        self.elapsed += millis;

        let remaining = ((TIMER_LEN - self.elapsed) as f32 / 1000.0).round() as i32;
        self.timer.borrow_mut().state.text = remaining.to_string();

        if remaining < 1 {
            self.revert(widget);
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let accept = Widget::with_theme(Button::empty(), "accept");
        accept.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let confirm = Widget::downcast_kind::<SaveOrRevertOptionsWindow>(&parent);
            confirm.accept(&parent);
        })));

        let revert = Widget::with_theme(Button::empty(), "revert");
        revert.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let confirm = Widget::downcast_kind::<SaveOrRevertOptionsWindow>(&parent);
            confirm.revert(&parent);
        })));

        vec![title, Rc::clone(&self.timer), accept, revert]
    }
}
