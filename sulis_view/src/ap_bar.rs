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

use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_core::io::event::ClickKind;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_module::Module;
use sulis_widgets::{Label};

pub const NAME: &str = "ap_bar";

pub struct ApBar {
    entity: Rc<RefCell<EntityState>>,
}

impl ApBar {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> Rc<RefCell<ApBar>> {
        Rc::new(RefCell::new(ApBar {
            entity,
        }))
    }
}

impl WidgetKind for ApBar {
    widget_kind!(NAME);

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

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        widget.borrow_mut().state.set_visible(GameState::is_current(&self.entity));

        let mut entity = self.entity.borrow_mut();
        entity.actor.listeners.add(ChangeListener::invalidate(NAME, widget));

        let rules = Module::rules();
        let ap_per_ball = rules.display_ap;
        let cur_ap = entity.actor.ap();
        let active_balls = cur_ap / ap_per_ball;
        let total_balls = rules.max_ap / ap_per_ball;

        let mut children = Vec::new();
        for i in 0..total_balls {
            let ball = Widget::with_theme(Label::empty(), "ball");
            ball.borrow_mut().state.set_active(i < active_balls);
            children.push(ball);
        }

        children
    }
}
