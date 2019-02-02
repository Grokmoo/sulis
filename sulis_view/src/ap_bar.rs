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
use sulis_module::{Module, OnTrigger};
use sulis_core::widgets::{ProgressBar};

use crate::RootView;

pub const NAME: &str = "ap_bar";

pub fn check_end_turn(widget: &Rc<RefCell<Widget>>) {
    let mut end_turn = false;
    {
        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        if let Some(entity) = mgr.current() {
            let entity = entity.borrow();
            if entity.is_party_member() &&
                entity.actor.ap() < entity.actor.get_move_ap_cost(1) &&
                !entity.actor.has_ap_to_attack() {

                end_turn = true;
            }
        }
    }

    if end_turn {
        let root = Widget::get_root(&widget);
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.end_turn();
    }
}

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
        let visible = GameState::is_current(&self.entity);

        let mut entity = self.entity.borrow_mut();

        widget.borrow_mut().state.set_visible(visible);

        let widget_ref = Rc::clone(widget);
        entity.actor.listeners.add(ChangeListener::new(NAME, Box::new(move |_| {
            widget_ref.borrow_mut().invalidate_children();
            let cb = OnTrigger::CheckEndTurn;
            let pc = GameState::player();
            GameState::add_ui_callback(vec![cb], &pc, &pc);
        })));

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |entity| {
            let bar = Widget::downcast_kind_mut::<ApBar>(&widget_ref);

            if let Some(entity) = entity {
                bar.entity = Rc::clone(entity);
            }

            widget_ref.borrow_mut().invalidate_children();
        })));

        let rules = Module::rules();
        let ap_per_ball = rules.display_ap;
        let total_balls = rules.max_ap / ap_per_ball;

        let mut children = Vec::new();
        let mut ap_left = entity.actor.ap();
        for _ in 0..total_balls {
            let frac;
            if ap_left > ap_per_ball {
                frac = 1.0;
                ap_left -= ap_per_ball;
            } else if ap_left == 0 {
                frac = 0.0;
            } else {
                frac = ap_left as f32 / ap_per_ball as f32;
                ap_left = 0;
            }

            let ball = ProgressBar::new(frac);
            let widget = Widget::with_theme(ball, "ball");
            children.push(widget);
        }

        children
    }
}
