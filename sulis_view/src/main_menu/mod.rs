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

use sulis_core::io::{InputAction, MainLoopUpdater};
use sulis_core::ui::*;
use sulis_widgets::{Button, ConfirmationWindow, Label, list_box, MutuallyExclusiveListBox};
use sulis_module::ModuleInfo;

pub struct MainMenuLoopUpdater {
    main_menu_view: Rc<RefCell<MainMenuView>>,
}

impl MainMenuLoopUpdater {
    pub fn new(main_menu_view: &Rc<RefCell<MainMenuView>>) -> MainMenuLoopUpdater {
        MainMenuLoopUpdater {
            main_menu_view: Rc::clone(main_menu_view),
        }
    }
}

impl MainLoopUpdater for MainMenuLoopUpdater {
    fn update(&self, _root: &Rc<RefCell<Widget>>) { }

    fn is_exit(&self) -> bool {
        self.main_menu_view.borrow().is_exit()
    }
}

pub struct MainMenuView {
    modules: Vec<ModuleInfo>,
}

impl MainMenuView {
    pub fn new(modules: Vec<ModuleInfo>) -> Rc<RefCell<MainMenuView>> {
        Rc::new(RefCell::new(MainMenuView {
            modules,
        }))
    }

    pub fn is_exit(&self) -> bool {
        EXIT.with(|exit| *exit.borrow())
    }

    pub fn get_selected_module(&self) -> Option<ModuleInfo> {
        SELECTED_MODULE.with(|m| {
            match m.borrow().as_ref() {
                None => None,
                Some(module) => Some(module.clone()),
            }
        })
    }
}

thread_local! {
    static EXIT: RefCell<bool> = RefCell::new(false);
    static SELECTED_MODULE: RefCell<Option<ModuleInfo>> = RefCell::new(None);
}

impl WidgetKind for MainMenuView {
    fn get_name(&self) -> &str {
        "root"
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        match key {
            Exit => {
                let exit_window = Widget::with_theme(
                    ConfirmationWindow::new(Callback::with(
                            Box::new(|| { EXIT.with(|exit| *exit.borrow_mut() = true); }))),
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
        let modules_title = Widget::with_theme(Label::empty(), "modules_title");
        let play = Widget::with_theme(Button::empty(), "play_button");
        let mut entries: Vec<list_box::Entry<ModuleInfo>> = Vec::new();

        let play_ref = Rc::clone(&play);
        let cb: Rc<Fn(Option<&list_box::Entry<ModuleInfo>>)> = Rc::new(move |active_entry| {
            play_ref.borrow_mut().state.set_enabled(active_entry.is_some());
        });
        for module in self.modules.iter() {
            let entry = list_box::Entry::new(module.clone(), None);
            entries.push(entry);
        }

        let list_box = MutuallyExclusiveListBox::with_callback(entries, cb);
        let list_box_ref = Rc::clone(&list_box);
        let modules_list = Widget::with_theme(list_box, "modules_list");

        play.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            let list_box_ref = list_box_ref.borrow();
            let entry = match list_box_ref.active_entry() {
                None => {
                    warn!("Play pressed with no active entry");
                    return;
                }, Some(entry) => entry,
            };

            EXIT.with(|exit| *exit.borrow_mut() = true);
            SELECTED_MODULE.with(|m| *m.borrow_mut() = Some(entry.item().clone()));
            info!("Selected module {}", entry.item().name);
        })));
        play.borrow_mut().state.disable();

        vec![title, modules_title, play, modules_list]
    }
}
