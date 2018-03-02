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

mod actor_picker;
use actor_picker::ActorPicker;

mod area_editor;
use area_editor::AreaEditor;

mod load_window;
use load_window::LoadWindow;

mod prop_picker;
use prop_picker::PropPicker;

mod save_window;
use save_window::SaveWindow;

mod shift_tiles_window;
use shift_tiles_window::ShiftTilesWindow;

mod tile_picker;
use tile_picker::TilePicker;

mod transition_window;
use transition_window::TransitionWindow;

#[macro_use] extern crate log;

extern crate sulis_core;
extern crate sulis_module;
extern crate sulis_widgets;

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::io::{InputAction, MainLoopUpdater};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, ConfirmationWindow, DropDown, list_box};

thread_local! {
    static EXIT: RefCell<bool> = RefCell::new(false);
}

pub struct EditorMainLoopUpdater { }

impl MainLoopUpdater for EditorMainLoopUpdater {
    fn update(&self, _root: &Rc<RefCell<Widget>>) { }

    fn is_exit(&self) -> bool {
        EXIT.with(|exit| *exit.borrow())
    }
}

const NAME: &str = "editor";

pub struct EditorView {
}

impl EditorView {
    pub fn new() -> Rc<RefCell<EditorView>> {
        Rc::new(RefCell::new(EditorView { }))
    }
}

impl WidgetKind for EditorView {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use InputAction::*;
        match key {
            Exit => {
                let exit_window = Widget::with_theme(ConfirmationWindow::new(Callback::with(
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
        debug!("Adding to editor widget");

        let tile_picker_kind = TilePicker::new();
        let actor_picker_kind = ActorPicker::new();
        let prop_picker_kind = PropPicker::new();
        let area_editor_kind = AreaEditor::new(&actor_picker_kind, &tile_picker_kind, &prop_picker_kind);

        let top_bar = Widget::empty("top_bar");
        {
            let mut entries: Vec<list_box::Entry<String>> = Vec::new();

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let save = list_box::Entry::new("Save".to_string(),
            Some(Callback::with_widget(Rc::new(move |widget| {
                let root = Widget::get_root(widget);
                let save_window = Widget::with_defaults(
                    SaveWindow::new(Rc::clone(&area_editor_kind_ref)));
                Widget::add_child_to(&root, save_window);

                let parent = Widget::get_parent(widget);
                parent.borrow_mut().mark_for_removal();
            }))));
            entries.push(save);

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let load = list_box::Entry::new("Load".to_string(),
            Some(Callback::with_widget(Rc::new(move |widget| {
                let root = Widget::get_root(widget);
                let load_window = Widget::with_defaults(LoadWindow::new(Rc::clone(&area_editor_kind_ref)));
                Widget::add_child_to(&root, load_window);

                let parent = Widget::get_parent(widget);
                parent.borrow_mut().mark_for_removal();
            }))));
            entries.push(load);

            let quit = list_box::Entry::new("Quit".to_string(),
            Some(Callback::with_widget(Rc::new(move |widget| {
                let root = Widget::get_root(widget);
                let exit_window = Widget::with_theme(ConfirmationWindow::new(Callback::with(
                            Box::new(|| { EXIT.with(|exit| *exit.borrow_mut() = true); }))),
                            "exit_confirmation_window");
                exit_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, exit_window);

                let parent = Widget::get_parent(widget);
                parent.borrow_mut().mark_for_removal();
            }))));
            entries.push(quit);

            let drop_down = DropDown::new(entries);
            let menu = Widget::with_theme(drop_down, "menu");

            let transitions = Widget::with_theme(Button::empty(), "transitions");

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            transitions.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let root = Widget::get_root(widget);
                let transition_window = Widget::with_defaults(
                    TransitionWindow::new(Rc::clone(&area_editor_kind_ref)));
                transition_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, transition_window);
            })));

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let shift_tiles = Widget::with_theme(Button::empty(), "shift_tiles");
            shift_tiles.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let root = Widget::get_root(widget);
                let shift_tiles_window = Widget::with_defaults(
                    ShiftTilesWindow::new(Rc::clone(&area_editor_kind_ref)));
                shift_tiles_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, shift_tiles_window);
            })));

            Widget::add_child_to(&top_bar, menu);
            Widget::add_child_to(&top_bar, transitions);
            Widget::add_child_to(&top_bar, shift_tiles);
        }

        let tile_picker = Widget::with_defaults(tile_picker_kind);
        let actor_picker = Widget::with_defaults(actor_picker_kind);
        let prop_picker = Widget::with_defaults(prop_picker_kind);
        actor_picker.borrow_mut().state.set_visible(false);
        prop_picker.borrow_mut().state.set_visible(false);

        let tiles = Widget::with_theme(Button::empty(), "tiles");

        let tile_picker_ref = Rc::clone(&tile_picker);
        let actor_picker_ref = Rc::clone(&actor_picker);
        let prop_picker_ref = Rc::clone(&prop_picker);
        let area_editor_ref = Rc::clone(&area_editor_kind);
        tiles.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            tile_picker_ref.borrow_mut().state.set_visible(true);
            actor_picker_ref.borrow_mut().state.set_visible(false);
            prop_picker_ref.borrow_mut().state.set_visible(false);
            area_editor_ref.borrow_mut().set_mode(area_editor::Mode::Tiles);
        })));

        let actors = Widget::with_theme(Button::empty(), "actors");

        let tile_picker_ref = Rc::clone(&tile_picker);
        let actor_picker_ref = Rc::clone(&actor_picker);
        let prop_picker_ref = Rc::clone(&prop_picker);
        let area_editor_ref = Rc::clone(&area_editor_kind);
        actors.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            tile_picker_ref.borrow_mut().state.set_visible(false);
            actor_picker_ref.borrow_mut().state.set_visible(true);
            prop_picker_ref.borrow_mut().state.set_visible(false);
            area_editor_ref.borrow_mut().set_mode(area_editor::Mode::Actors);
        })));

        let props = Widget::with_theme(Button::empty(), "props");

        let tile_picker_ref = Rc::clone(&tile_picker);
        let actor_picker_ref = Rc::clone(&actor_picker);
        let prop_picker_ref = Rc::clone(&prop_picker);
        let area_editor_ref = Rc::clone(&area_editor_kind);
        props.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            tile_picker_ref.borrow_mut().state.set_visible(false);
            actor_picker_ref.borrow_mut().state.set_visible(false);
            prop_picker_ref.borrow_mut().state.set_visible(true);
            area_editor_ref.borrow_mut().set_mode(area_editor::Mode::Props);
        })));

        let area_editor = Widget::with_defaults(area_editor_kind);

        Widget::add_child_to(&top_bar, tiles);
        Widget::add_child_to(&top_bar, actors);
        Widget::add_child_to(&top_bar, props);

        vec![area_editor, actor_picker, tile_picker, prop_picker, top_bar]
    }
}
