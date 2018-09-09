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

use sulis_core::ui::*;
use sulis_widgets::{Button, Label, list_box, MutuallyExclusiveListBox, TextArea};
use sulis_module::{ModuleInfo};

use main_menu::MainMenu;

pub struct ModuleSelector {
    modules: Vec<ModuleInfo>,
}

impl ModuleSelector {
    pub fn new(modules: Vec<ModuleInfo>) -> Rc<RefCell<ModuleSelector>> {
        Rc::new(RefCell::new(ModuleSelector {
            modules,
        }))
    }
}

impl WidgetKind for ModuleSelector {
    widget_kind!("module_selector");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let modules_title = Widget::with_theme(Label::empty(), "modules_title");
        let play = Widget::with_theme(Button::empty(), "play_button");
        let mut entries: Vec<list_box::Entry<ModuleInfo>> = Vec::new();

        let details = Widget::with_theme(TextArea::empty(), "details");

        let details_ref = Rc::clone(&details);
        let play_ref = Rc::clone(&play);
        let cb: Rc<Fn(Option<&list_box::Entry<ModuleInfo>>)> = Rc::new(move |active_entry| {
            play_ref.borrow_mut().state.set_enabled(active_entry.is_some());
            if let Some(entry) = active_entry {
                details_ref.borrow_mut().state.add_text_arg("description", &entry.item().description);
            } else {
                details_ref.borrow_mut().state.clear_text_args();
            }
            details_ref.borrow_mut().invalidate_layout();
        });
        for module in self.modules.iter() {
            let entry = list_box::Entry::new(module.clone(), None);
            entries.push(entry);
        }

        let list_box = MutuallyExclusiveListBox::with_callback(entries, cb);
        let list_box_ref = Rc::clone(&list_box);
        let modules_list = Widget::with_theme(list_box, "modules_list");

        play.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let list_box_ref = list_box_ref.borrow();
            let entry = match list_box_ref.active_entry() {
                None => {
                    warn!("Play pressed with no active entry");
                    return;
                }, Some(entry) => entry,
            };

            let module = entry.item().clone();

            let root = Widget::get_root(&widget);
            let menu = Widget::downcast_kind_mut::<MainMenu>(&root);
            menu.load_module(&module.dir);
            root.borrow_mut().invalidate_children();
        })));
        play.borrow_mut().state.disable();

        vec![title, modules_title, play, modules_list, details]
    }
}
