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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::rc::Rc;
use std::cell::RefCell;

use grt::io::{InputAction, MainLoopUpdater};
use grt::ui::*;

use module::ModuleInfo;

pub struct MainMenuLoopUpdater {
    main_menu_view: Rc<MainMenuView>,
}

impl MainMenuLoopUpdater {
    pub fn new(main_menu_view: &Rc<MainMenuView>) -> MainMenuLoopUpdater {
        MainMenuLoopUpdater {
            main_menu_view: Rc::clone(main_menu_view),
        }
    }
}

impl MainLoopUpdater for MainMenuLoopUpdater {
    fn update(&self) { }

    fn is_exit(&self) -> bool {
        self.main_menu_view.is_exit()
    }
}

pub struct MainMenuView {
    modules: Vec<ModuleInfo>,
}

impl MainMenuView {
    pub fn new(modules: Vec<ModuleInfo>) -> Rc<MainMenuView> {
        Rc::new(MainMenuView {
            modules,
        })
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

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use grt::io::InputAction::*;
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

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to main menu widget");

        let title = Widget::with_theme(Label::empty(), "title");
        let modules_title = Widget::with_theme(Label::empty(), "modules_title");
        let play = Widget::with_theme(Button::empty(), "play_button");
        let mut entries: Vec<list_box::Entry<ModuleInfo>> = Vec::new();

        let play_ref = Rc::clone(&play);
        let cb: Callback = Callback::new(Rc::new(move |widget| {
            let parent = Widget::get_parent(widget);
            let cur_state = widget.borrow_mut().state.animation_state.contains(animation_state::Kind::Active);
            if !cur_state {
                for child in parent.borrow_mut().children.iter() {
                    child.borrow_mut().state.animation_state.remove(animation_state::Kind::Active);
                }
                play_ref.borrow_mut().state.enable();
            } else {
                play_ref.borrow_mut().state.disable();
            }
            widget.borrow_mut().state.animation_state.toggle(animation_state::Kind::Active);
        }));
        for module in self.modules.iter() {
            let entry = list_box::Entry::new(module.clone(), Some(cb.clone()));
            entries.push(entry);
        }

        let list_box = ListBox::new(entries);
        // using Rc::clone complains of type mismatch here
        let modules_list = Widget::with_theme(list_box.clone(), "modules_list");

        let modules_list_ref = Rc::clone(&modules_list);
        play.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_| {
            for (index, child) in modules_list_ref.borrow().children.iter().enumerate() {
                if child.borrow().state.animation_state.contains(animation_state::Kind::Active) {
                    let entry = list_box.get(index).unwrap();
                    EXIT.with(|exit| *exit.borrow_mut() = true);
                    SELECTED_MODULE.with(|m| *m.borrow_mut() = Some(entry.item().clone()));
                    info!("Found active module {}", entry.item().name );
                }
            }
        })));
        play.borrow_mut().state.disable();

        vec![title, modules_title, play, modules_list]
    }
}
