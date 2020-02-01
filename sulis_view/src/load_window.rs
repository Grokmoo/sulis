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
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{
    Button, ConfirmationWindow, Label, ScrollDirection, ScrollPane, TextArea,
};
use sulis_state::save_file::{delete_save, get_available_save_files, load_state};
use sulis_state::{NextGameStep, SaveFileMetaData, SaveState};

use crate::{main_menu::MainMenu, LoadingScreen, RootView};

const NAME: &str = "load_window";

pub struct LoadWindow {
    accept: Rc<RefCell<Widget>>,
    delete: Rc<RefCell<Widget>>,
    pub(crate) cancel: Rc<RefCell<Widget>>,
    entries: Vec<SaveFileMetaData>,
    selected_entry: Option<usize>,
    main_menu_mode: bool,
}

impl LoadWindow {
    pub fn new(main_menu_mode: bool) -> Rc<RefCell<LoadWindow>> {
        let accept = Widget::with_theme(Button::empty(), "accept");
        let cancel = Widget::with_theme(Button::empty(), "cancel");
        let delete = Widget::with_theme(Button::empty(), "delete");
        let entries = match get_available_save_files() {
            Ok(files) => files,
            Err(e) => {
                warn!("Unable to read saved files");
                warn!("{}", e);
                Vec::new()
            }
        };

        Rc::new(RefCell::new(LoadWindow {
            accept,
            delete,
            cancel,
            entries,
            selected_entry: None,
            main_menu_mode,
        }))
    }

    pub fn load(&self, root: &Rc<RefCell<Widget>>) {
        let index = match self.selected_entry {
            None => return,
            Some(index) => index,
        };

        match load_state(&self.entries[index]) {
            Err(e) => {
                error!("Error reading game state");
                error!("{}", e);
            }
            Ok(state) => {
                self.set_load_step(state, root);
            }
        }
    }

    pub fn set_load_step(&self, save_state: SaveState, root: &Rc<RefCell<Widget>>) {
        // TODO remove the bool flag passed in the constructor
        if self.main_menu_mode {
            let main_menu = Widget::kind_mut::<MainMenu>(root);
            main_menu.next_step = Some(NextGameStep::LoadCampaign { save_state });
        } else {
            let root_view = Widget::kind_mut::<RootView>(root);
            root_view.next_step = Some(NextGameStep::LoadCampaign { save_state });
        }

        let loading_screen = Widget::with_defaults(LoadingScreen::new());
        loading_screen.borrow_mut().state.set_modal(true);
        Widget::add_child_to(&root, loading_screen);
    }

    pub fn delete_save(&mut self) {
        let index = match self.selected_entry {
            None => return,
            Some(index) => index,
        };

        match delete_save(&self.entries[index]) {
            Err(e) => {
                error!("Error deleting save");
                error!("{}", e);
            }
            Ok(()) => (),
        }

        self.entries.remove(index);
    }

    fn set_button_state(&self) {
        self.delete
            .borrow_mut()
            .state
            .set_enabled(self.selected_entry.is_some());

        let accept_enabled = match self.selected_entry {
            None => false,
            Some(index) => self.entries[index].error.is_none(),
        };

        self.accept.borrow_mut().state.set_enabled(accept_enabled);
    }
}

impl WidgetKind for LoadWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        self.cancel
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<LoadWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let load_window_widget_ref = Rc::clone(widget);
        let delete_cb = Callback::new(Rc::new(move |widget, _| {
            load_window_widget_ref.borrow_mut().invalidate_children();

            let load_window = Widget::kind_mut::<LoadWindow>(&load_window_widget_ref);
            load_window.delete_save();
            load_window.selected_entry = None;
            load_window.set_button_state();

            let (parent, _) = Widget::parent::<ConfirmationWindow>(widget);
            parent.borrow_mut().mark_for_removal();
        }));

        self.delete
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let root = Widget::get_root(widget);
                let conf_window = Widget::with_theme(
                    ConfirmationWindow::new(delete_cb.clone()),
                    "delete_save_confirmation",
                );
                conf_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, conf_window);
            })));

        self.accept
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, load_window) = Widget::parent_mut::<LoadWindow>(widget);
                let root = Widget::get_root(&parent);
                load_window.load(&root);

                parent.borrow_mut().mark_for_removal();
            })));

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        let entries = Widget::with_theme(scrollpane.clone(), "entries");

        for (index, ref meta) in self.entries.iter().enumerate() {
            let text_area = Widget::with_defaults(TextArea::empty());
            text_area
                .borrow_mut()
                .state
                .add_text_arg("player_name", &meta.player_name);
            text_area
                .borrow_mut()
                .state
                .add_text_arg("datetime", &meta.datetime);
            text_area
                .borrow_mut()
                .state
                .add_text_arg("current_area_name", &meta.current_area_name);
            if meta.error.is_some() {
                text_area
                    .borrow_mut()
                    .state
                    .add_text_arg("error", &meta.error.as_ref().unwrap());
            }

            let widget = Widget::with_theme(Button::empty(), "entry");
            widget
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, load_window) = Widget::parent_mut::<LoadWindow>(widget);
                    parent.borrow_mut().invalidate_layout();
                    load_window.selected_entry = Some(index);
                    load_window.set_button_state();

                    let content = Widget::direct_parent(widget);
                    for child in content.borrow().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);
                })));

            if let Some(selected_index) = self.selected_entry {
                if selected_index == index {
                    widget.borrow_mut().state.set_active(true);
                }
            }

            Widget::add_child_to(&widget, text_area);
            scrollpane.borrow().add_to_content(widget);
        }

        self.set_button_state();

        vec![
            self.cancel.clone(),
            self.delete.clone(),
            self.accept.clone(),
            title,
            entries,
        ]
    }
}
