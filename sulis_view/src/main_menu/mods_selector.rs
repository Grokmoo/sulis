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
use std::path::PathBuf;

use sulis_core::ui::*;
use sulis_core::util::ActiveResources;
use sulis_core::widgets::{Button, Label, ScrollPane, TextArea};
use sulis_module::{ModificationInfo};
use sulis_state::NextGameStep;

use crate::main_menu::{MainMenu};
use crate::LoadingScreen;

pub struct ModsSelector {
    available_mods: Vec<ModificationInfo>,
    active_mods: Vec<ModificationInfo>,
}

impl ModsSelector {
    pub fn new(mods: Vec<ModificationInfo>) -> Rc<RefCell<ModsSelector>> {
        let mut available_mods = Vec::new();
        let mut active_mods = Vec::new();

        let active_resources = ActiveResources::read();
        let active: Vec<_> = active_resources.mods.iter().map(|path| PathBuf::from(&path)).collect();

        for modif in mods {
            let mod_path = PathBuf::from(&modif.dir);

            if active.contains(&mod_path) {
                active_mods.push(modif);
            } else {
                available_mods.push(modif);
            }
        }

        Rc::new(RefCell::new(ModsSelector {
            available_mods,
            active_mods,
        }))
    }
}

impl WidgetKind for ModsSelector {
    widget_kind!("mods_selector");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let available_title = Widget::with_theme(Label::empty(), "available_title");
        let active_title = Widget::with_theme(Label::empty(), "active_title");

        let available_pane = ScrollPane::new();
        let available = Widget::with_theme(available_pane.clone(), "available");

        let active_pane = ScrollPane::new();
        let active = Widget::with_theme(active_pane.clone(), "active");

        self.available_mods.sort_by(|ref a, ref b| a.name.cmp(&b.name));

        let len = self.available_mods.len();
        for (index, modif) in self.available_mods.iter().enumerate() {
            let widget = Widget::with_defaults(ModPane::new(modif.clone(), false, index, len));
            available_pane.borrow().add_to_content(widget);
        }

        let len = self.active_mods.len();
        for (index, modif) in self.active_mods.iter().enumerate() {
            let widget = Widget::with_defaults(ModPane::new(modif.clone(), true, index, len));
            active_pane.borrow().add_to_content(widget);
        }

        let clear = Widget::with_theme(Button::empty(), "clear");
        clear.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let (parent, sel) = Widget::parent_mut::<ModsSelector>(widget);

             for modif in sel.active_mods.drain(..) {
                 sel.available_mods.push(modif);
             }

            parent.borrow_mut().invalidate_children();
        })));

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let (root, menu) = Widget::parent_mut::<MainMenu>(widget);
            menu.reset();
            root.borrow_mut().invalidate_children();
        })));

        let apply = Widget::with_theme(Button::empty(), "apply");
        apply.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let (_, sel) = Widget::parent_mut::<ModsSelector>(widget);

            let mut resources = ActiveResources::read();
            resources.mods.clear();
            for modif in sel.active_mods.iter() {
                resources.mods.push(modif.dir.to_string());
            }

            resources.write();

            let (root, menu) = Widget::parent_mut::<MainMenu>(widget);
            menu.next_step = Some(NextGameStep::MainMenuReloadResources);

            let loading_screen = Widget::with_defaults(LoadingScreen::new());
            loading_screen.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, loading_screen);
        })));

        vec![title, available_title, active_title, available, active, clear, cancel, apply]
    }
}

pub struct ModPane {
    modif: ModificationInfo,
    active: bool,
    index: usize,
    vec_len: usize,
}

impl ModPane {
    pub fn new(modif: ModificationInfo, active: bool, index: usize,
               vec_len: usize) -> Rc<RefCell<ModPane>> {
        Rc::new(RefCell::new(ModPane {
            modif,
            active,
            index,
            vec_len,
        }))
    }
}

impl WidgetKind for ModPane {
    widget_kind!("mod_pane");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let description = Widget::with_theme(TextArea::empty(), "description");

        {
            let state = &mut description.borrow_mut().state;
            state.add_text_arg("name", &self.modif.name);
            state.add_text_arg("description", &self.modif.description);
            state.add_text_arg("dir", &self.modif.dir);
        }

        let toggle = Widget::with_theme(Button::empty(), "toggle");

        let active = self.active;
        let index = self.index;
        toggle.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let (parent, sel) = Widget::parent_mut::<ModsSelector>(widget);

            if active {
                let modif = sel.active_mods.remove(index);
                sel.available_mods.push(modif);
            } else {
                let modif = sel.available_mods.remove(index);
                sel.active_mods.push(modif);
            }

            parent.borrow_mut().invalidate_children();
        })));

        let up = Widget::with_theme(Button::empty(), "up");
        let active = self.active;
        let index = self.index;
        up.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            if index == 0 { return; }

            let (parent, sel) = Widget::parent_mut::<ModsSelector>(widget);

            let vec = if active {
                &mut sel.active_mods
            } else {
                &mut sel.available_mods
            };

            let modif = vec.remove(index);
            vec.insert(index - 1, modif);

            parent.borrow_mut().invalidate_children();
        })));

        let down = Widget::with_theme(Button::empty(), "down");
        let active = self.active;
        let index = self.index;
        let len = self.vec_len;
        down.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            if index == len - 1 { return; }

            let (parent, sel) = Widget::parent_mut::<ModsSelector>(widget);

            let vec = if active {
                &mut sel.active_mods
            } else {
                &mut sel.available_mods
            };

            let modif = vec.remove(index);
            vec.insert(index + 1, modif);

            parent.borrow_mut().invalidate_children();
        })));

        if !self.active {
            up.borrow_mut().state.set_visible(false);
            down.borrow_mut().state.set_visible(false);
        }

        if index == 0 { up.borrow_mut().state.set_enabled(false); }
        if index == self.vec_len - 1 { down.borrow_mut().state.set_enabled(false); }

        vec![description, toggle, up, down]
    }
}
