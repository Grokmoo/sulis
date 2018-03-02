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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, Spinner};

use {area_editor, AreaEditor};

pub const NAME: &str = "shift_tiles_window";

pub struct ShiftTilesWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
}

impl ShiftTilesWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<ShiftTilesWindow>> {
        Rc::new(RefCell::new(ShiftTilesWindow {
            area_editor,
        }))
    }
}

impl WidgetKind for ShiftTilesWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let x_label = Widget::with_theme(Label::empty(), "x_label");
        let y_label = Widget::with_theme(Label::empty(), "y_label");

        let x_spinner = Spinner::new(0, -99, 99);
        let y_spinner = Spinner::new(0, -99, 99);
        let area_editor_ref = Rc::clone(&self.area_editor);

        let x_spinner_widget = Widget::with_theme(x_spinner.clone(), "x_spinner");
        let y_spinner_widget = Widget::with_theme(y_spinner.clone(), "y_spinner");

        let accept = Widget::with_theme(Button::empty(), "apply_button");
        accept.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let delta_x = x_spinner.borrow().value();
            let delta_y = y_spinner.borrow().value();
            info!("Shifting tiles in area by {},{}", delta_x, delta_y);
            shift_tiles(&mut *area_editor_ref.borrow_mut(), delta_x, delta_y);

            let parent = Widget::get_parent(&widget);
            parent.borrow_mut().mark_for_removal();
        })));

        vec![title, close, x_label, y_label, x_spinner_widget, y_spinner_widget, accept]
    }
}

fn shift_tiles(area_editor: &mut AreaEditor, delta_x: i32, delta_y: i32) {
    for &mut (_, ref mut layer) in area_editor.tiles.iter_mut() {
        for &mut (ref mut point, ref tile) in layer.iter_mut() {
            if point.x + delta_x < 0
                || point.y + delta_y < 0
                || point.x + delta_x + tile.width > area_editor::MAX_AREA_SIZE
                || point.y + delta_y + tile.height > area_editor::MAX_AREA_SIZE {
                warn!("Invalid tile shift parameters: {},{}", delta_x, delta_y);
                return;
            }
        }
    }

    for &mut (_, ref mut layer) in area_editor.tiles.iter_mut() {
        for &mut (ref mut point, _) in layer.iter_mut() {
            point.x += delta_x;
            point.y += delta_y;
        }
    }
}
