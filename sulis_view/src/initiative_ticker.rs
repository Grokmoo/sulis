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

use sulis_core::io::event::ClickKind;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_state::{ChangeListener, GameState};
use sulis_widgets::Label;

pub const NAME: &str = "initiative_ticker";

pub struct InitiativeTicker { }

impl InitiativeTicker {
    pub fn new() -> Rc<RefCell<InitiativeTicker>> {
        Rc::new(RefCell::new(InitiativeTicker { }))
    }
}

impl WidgetKind for InitiativeTicker {
    widget_kind!(NAME);

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        true
    }

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(&mut self, _widget: &Rc<RefCell<Widget>>, _kind: ClickKind,
                     _delta_x: f32, _delta_y: f32) -> bool {
        false
    }

    fn on_remove(&mut self) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        area_state.turn_timer.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        let timer = &mut area_state.turn_timer;
        timer.listeners.add(ChangeListener::invalidate(NAME, widget));

        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();
        let mut first = true;
        for entity in timer.active_iter() {
            let theme = match first {
                true => "current_entry",
                false => "entry",
            };
            let widget = Widget::with_theme(Label::new(&entity.borrow().actor.actor.name),
                                            theme);
            widgets.push(widget);
            first = false;
        }

        widgets
    }
}
