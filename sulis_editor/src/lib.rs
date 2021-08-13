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

#![allow(clippy::manual_range_contains)]

mod actor_picker;
use crate::actor_picker::ActorPicker;

mod actor_creator_window;
use crate::actor_creator_window::ActorCreatorWindow;

mod area_editor;
use crate::area_editor::AreaEditor;

mod area_model;
use crate::area_model::AreaModel;

mod elev_picker;
use crate::elev_picker::ElevPicker;

mod encounter_picker;
use crate::encounter_picker::EncounterPicker;

mod feature_picker;
use crate::feature_picker::FeaturePicker;

mod load_window;
use crate::load_window::LoadWindow;

mod pass_picker;
use crate::pass_picker::PassPicker;

mod prop_picker;
use crate::prop_picker::PropPicker;

mod save_window;
use crate::save_window::SaveWindow;

mod shift_tiles_window;
use crate::shift_tiles_window::ShiftTilesWindow;

mod terrain_picker;
use crate::terrain_picker::TerrainPicker;

mod tile_picker;
use crate::tile_picker::TilePicker;

mod transition_window;
use crate::transition_window::TransitionWindow;

mod trigger_picker;
use crate::trigger_picker::TriggerPicker;

mod vis_picker;
use crate::vis_picker::VisPicker;

mod wall_picker;
use crate::wall_picker::WallPicker;

#[macro_use]
extern crate log;

use std::any::Any;
use std::cell::{RefCell, Cell};
use std::rc::Rc;

use sulis_core::io::{GraphicsRenderer, InputActionKind, ControlFlowUpdater};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::{Offset, Scale};
use sulis_core::widgets::{list_box, Button, ConfirmationWindow, DropDown};

thread_local! {
    static EXIT: Cell<bool> = Cell::new(false);
}

pub struct EditorControlFlowUpdater {
    root: Rc<RefCell<Widget>>,
}

impl EditorControlFlowUpdater {
    pub fn new(root: Rc<RefCell<Widget>>) -> EditorControlFlowUpdater {
        EditorControlFlowUpdater {
            root,
        }
    }
}

impl ControlFlowUpdater for EditorControlFlowUpdater {
    fn update(&mut self, millis: u32) -> Rc<RefCell<Widget>> {
        if let Err(e) = Widget::update(&self.root, millis) {
            error!("There was a fatal error updating the UI tree state.");
            error!("{}", e);
            EXIT.with(|exit| exit.set(true));
        }

        self.root()
    }

    fn recreate_window(&mut self) -> bool { false }

    fn root(&self) -> Rc<RefCell<Widget>> {
        Rc::clone(&self.root)
    }

    fn is_exit(&self) -> bool {
        EXIT.with(|exit| exit.get())
    }
}

pub trait EditorMode: WidgetKind {
    fn draw_mode(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        model: &AreaModel,
        offset: Offset,
        scale: Scale,
        millis: u32,
    );

    fn cursor_size(&self) -> (i32, i32);

    fn mouse_move(&mut self, model: &mut AreaModel, x: i32, y: i32);

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32);

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32);

    fn mouse_scroll(&mut self, _model: &mut AreaModel, _delta: i32) {}
}

const NAME: &str = "editor";

pub struct EditorView {}

impl EditorView {
    pub fn new() -> Rc<RefCell<EditorView>> {
        Rc::new(RefCell::new(EditorView {}))
    }
}

impl WidgetKind for EditorView {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputActionKind) -> bool {
        use crate::InputActionKind::*;
        match key {
            Back => {
                let exit_window = Widget::with_theme(
                    ConfirmationWindow::new(Callback::with(Box::new(|| {
                        EXIT.with(|exit| exit.set(true));
                    }))),
                    "exit_confirmation_window",
                );
                exit_window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(widget, exit_window);
            }
            _ => return false,
        }

        true
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to editor widget");

        let area_editor_kind = AreaEditor::new();

        let top_bar = Widget::empty("top_bar");
        {
            let mut entries: Vec<list_box::Entry<String>> = Vec::new();

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let new = list_box::Entry::new(
                "New".to_string(),
                Some(Callback::with_widget(Rc::new(move |widget| {
                    area_editor_kind_ref.borrow_mut().clear_area();
                    let parent = Widget::direct_parent(widget);
                    parent.borrow_mut().mark_for_removal();
                }))),
            );
            entries.push(new);

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let save = list_box::Entry::new(
                "Save".to_string(),
                Some(Callback::with_widget(Rc::new(move |widget| {
                    let root = Widget::get_root(widget);
                    let save_window =
                        Widget::with_defaults(SaveWindow::new(Rc::clone(&area_editor_kind_ref)));
                    Widget::add_child_to(&root, save_window);

                    let parent = Widget::direct_parent(widget);
                    parent.borrow_mut().mark_for_removal();
                }))),
            );
            entries.push(save);

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let load = list_box::Entry::new(
                "Load".to_string(),
                Some(Callback::with_widget(Rc::new(move |widget| {
                    let root = Widget::get_root(widget);
                    let load_window =
                        Widget::with_defaults(LoadWindow::new(Rc::clone(&area_editor_kind_ref)));
                    Widget::add_child_to(&root, load_window);

                    let parent = Widget::direct_parent(widget);
                    parent.borrow_mut().mark_for_removal();
                }))),
            );
            entries.push(load);

            let quit = list_box::Entry::new(
                "Quit".to_string(),
                Some(Callback::with_widget(Rc::new(move |widget| {
                    let root = Widget::get_root(widget);
                    let exit_window = Widget::with_theme(
                        ConfirmationWindow::new(Callback::with(Box::new(|| {
                            EXIT.with(|exit| exit.set(true));
                        }))),
                        "exit_confirmation_window",
                    );
                    exit_window.borrow_mut().state.set_modal(true);
                    Widget::add_child_to(&root, exit_window);

                    let parent = Widget::direct_parent(widget);
                    parent.borrow_mut().mark_for_removal();
                }))),
            );
            entries.push(quit);

            let drop_down = DropDown::new(entries, "menu_list");
            let menu = Widget::with_theme(drop_down, "menu");

            let transitions = Widget::with_theme(Button::empty(), "transitions");

            let top_bar_ref = Rc::clone(&top_bar);
            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            transitions
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let root = Widget::get_root(widget);
                    let transition_window = Widget::with_defaults(TransitionWindow::new(
                        Rc::clone(&area_editor_kind_ref),
                        Rc::clone(&top_bar_ref),
                    ));
                    Widget::add_child_to(&root, transition_window);
                })));

            let area_editor_kind_ref = Rc::clone(&area_editor_kind);
            let shift_tiles = Widget::with_theme(Button::empty(), "shift_tiles");
            shift_tiles
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let root = Widget::get_root(widget);
                    let shift_tiles_window = Widget::with_defaults(ShiftTilesWindow::new(
                        Rc::clone(&area_editor_kind_ref),
                    ));
                    shift_tiles_window.borrow_mut().state.set_modal(true);
                    Widget::add_child_to(&root, shift_tiles_window);
                })));

            let actor_creator = Widget::with_theme(Button::empty(), "actor_creator");
            actor_creator
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let root = Widget::get_root(widget);
                    let window = Widget::with_defaults(ActorCreatorWindow::new());
                    window.borrow_mut().state.set_modal(true);
                    Widget::add_child_to(&root, window);
                })));

            Widget::add_child_to(&top_bar, menu);
            Widget::add_child_to(&top_bar, transitions);
            Widget::add_child_to(&top_bar, shift_tiles);
            Widget::add_child_to(&top_bar, actor_creator);
        }

        let tile_picker_kind = TilePicker::new();
        let terrain_picker_kind = TerrainPicker::new();
        let wall_picker_kind = WallPicker::new();
        let feature_picker_kind = FeaturePicker::new();
        let actor_picker_kind = ActorPicker::new();
        let prop_picker_kind = PropPicker::new();
        let elev_picker_kind = ElevPicker::new();
        let encounter_picker_kind = EncounterPicker::new();
        let trigger_picker_kind = TriggerPicker::new();
        let pass_picker_kind = PassPicker::new();
        let vis_picker_kind = VisPicker::new();

        let pickers = vec![
            Widget::with_defaults(tile_picker_kind.clone()),
            Widget::with_defaults(terrain_picker_kind.clone()),
            Widget::with_defaults(wall_picker_kind.clone()),
            Widget::with_defaults(feature_picker_kind.clone()),
            Widget::with_defaults(actor_picker_kind.clone()),
            Widget::with_defaults(prop_picker_kind.clone()),
            Widget::with_defaults(elev_picker_kind.clone()),
            Widget::with_defaults(encounter_picker_kind.clone()),
            Widget::with_defaults(trigger_picker_kind.clone()),
            Widget::with_defaults(pass_picker_kind.clone()),
            Widget::with_defaults(vis_picker_kind.clone()),
        ];
        for picker in pickers.iter() {
            picker.borrow_mut().state.set_visible(false);
        }

        let picker_kinds: Vec<Rc<RefCell<dyn EditorMode>>> = vec![
            tile_picker_kind,
            terrain_picker_kind,
            wall_picker_kind,
            feature_picker_kind,
            actor_picker_kind,
            prop_picker_kind,
            elev_picker_kind,
            encounter_picker_kind,
            trigger_picker_kind,
            pass_picker_kind,
            vis_picker_kind,
        ];

        let names = vec![
            "Tiles",
            "Terrain",
            "Walls",
            "Features",
            "Actors",
            "Props",
            "Elevation",
            "Encounters",
            "Triggers",
            "Passability",
            "Visibility",
        ];

        // Any new pickers need to be added in all 3 places
        assert!(names.len() == picker_kinds.len());
        assert!(names.len() == pickers.len());

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, name) in names.into_iter().enumerate() {
            let pickers_ref = pickers.clone();
            let picker_kinds_ref = picker_kinds.clone();
            let area_editor_ref = Rc::clone(&area_editor_kind);
            entries.push(list_box::Entry::new(
                name.to_string(),
                Some(Callback::new(Rc::new(move |widget, _| {
                    pickers_ref
                        .iter()
                        .for_each(|p| p.borrow_mut().state.set_visible(false));
                    pickers_ref[index].borrow_mut().state.set_visible(true);
                    pickers_ref[index].borrow_mut().invalidate_children();
                    area_editor_ref
                        .borrow_mut()
                        .set_editor(picker_kinds_ref[index].clone());

                    let parent = Widget::direct_parent(widget);
                    parent.borrow_mut().mark_for_removal();
                }))),
            ));
        }
        let drop_down = DropDown::new(entries, "modes_list");
        let modes = Widget::with_theme(drop_down, "modes");
        Widget::add_child_to(&top_bar, modes);

        let area_editor = Widget::with_defaults(area_editor_kind);

        let mut children = Vec::with_capacity(pickers.len() + 2);
        children.push(area_editor);
        children.extend_from_slice(&pickers);
        children.push(top_bar);

        children
    }
}
