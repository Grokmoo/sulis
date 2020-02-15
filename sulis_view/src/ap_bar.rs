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
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::io::event::ClickKind;
use sulis_core::ui::{Widget, WidgetKind, RcRfc};
use sulis_core::widgets::ProgressBar;
use sulis_module::{Module, OnTrigger};
use sulis_state::{ChangeListener, EntityState, GameState};

use crate::RootView;

pub const NAME: &str = "ap_bar";

pub fn check_end_turn(widget: &RcRfc<Widget>) {
    let mut end_turn = false;
    {
        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();

        let entity = match mgr.current() {
            None => return,
            Some(entity) => entity,
        };
        let entity = entity.borrow();
        if !entity.is_party_member() {
            return;
        }

        if !entity.actor.started_turn_with_no_ap_for_actions()
            && !entity.actor.has_ap_for_any_action()
        {
            end_turn = true;
            info!("{} has no AP left.  Ending turn.", entity.actor.actor.name);
        }
    }

    if end_turn {
        let (_, view) = Widget::parent_mut::<RootView>(widget);
        view.end_turn();
    }
}

pub struct ApBar {
    entity: RcRfc<EntityState>,
}

impl ApBar {
    pub fn new(entity: RcRfc<EntityState>) -> RcRfc<ApBar> {
        Rc::new(RefCell::new(ApBar { entity }))
    }
}

impl WidgetKind for ApBar {
    widget_kind!(NAME);

    fn on_mouse_press(&mut self, widget: &RcRfc<Widget>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &RcRfc<Widget>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(
        &mut self,
        _widget: &RcRfc<Widget>,
        _kind: ClickKind,
        _delta_x: f32,
        _delta_y: f32,
    ) -> bool {
        false
    }

    fn on_add(&mut self, widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let visible = GameState::is_current(&self.entity);

        let mut entity = self.entity.borrow_mut();

        widget.borrow_mut().state.set_visible(visible);

        let widget_ref = Rc::clone(widget);
        let player_ref = GameState::player();
        entity.actor.listeners.add(ChangeListener::new(
            NAME,
            Box::new(move |_| {
                widget_ref.borrow_mut().invalidate_children();
                let cb = OnTrigger::CheckEndTurn;
                GameState::add_ui_callback(vec![cb], &player_ref, &player_ref);
            }),
        ));

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(
            NAME,
            Box::new(move |entity| {
                let bar = Widget::kind_mut::<ApBar>(&widget_ref);

                if let Some(entity) = entity {
                    bar.entity = Rc::clone(entity);
                }

                widget_ref.borrow_mut().invalidate_children();
            }),
        ));

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
