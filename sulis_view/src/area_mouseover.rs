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

use sulis_core::ui::{Widget, WidgetKind, WidgetState};
use sulis_core::io::{GraphicsRenderer, event::ClickKind};
use sulis_core::util::Point;
use sulis_widgets::{TextArea};
use sulis_state::{ChangeListener, EntityState, GameState};

const NAME: &'static str = "area_mouseover";

enum Kind {
    Entity(Rc<RefCell<EntityState>>),
    Prop(usize),
    Transition(String),
}

impl PartialEq for AreaMouseover {
    fn eq(&self, other: &AreaMouseover) -> bool {
        match &self.kind {
            Kind::Entity(ref entity) => {
                match &other.kind {
                    Kind::Entity(ref other_entity) => Rc::ptr_eq(entity, other_entity),
                    _ => false,
                }
            },
            Kind::Prop(index) => {
                match &other.kind {
                    Kind::Prop(other_index) => other_index == index,
                    _ => false,
                }
            },
            Kind::Transition(ref name) => {
                match &other.kind {
                    Kind::Transition(ref other_name) => other_name == name,
                    _ => false,
                }
            }
        }
    }
}

pub struct AreaMouseover {
    kind: Kind,
    text_area: Rc<RefCell<TextArea>>,
}

impl AreaMouseover {
    pub fn new_entity(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<AreaMouseover>> {
        AreaMouseover::new(Kind::Entity(Rc::clone(entity)))
    }

    pub fn new_prop(index: usize) -> Rc<RefCell<AreaMouseover>> {
        AreaMouseover::new(Kind::Prop(index))
    }

    pub fn new_transition(name: &str) -> Rc<RefCell<AreaMouseover>> {
        AreaMouseover::new(Kind::Transition(name.to_string()))
    }

    fn new(kind: Kind) -> Rc<RefCell<AreaMouseover>> {
        Rc::new(RefCell::new(AreaMouseover {
            kind,
            text_area: TextArea::empty(),
        }))
    }

    fn set_text_args(&self, state: &mut WidgetState) -> bool {
        state.clear_text_args();

        match self.kind {
            Kind::Entity(ref entity) => {
                let actor = &entity.borrow().actor;
                state.add_text_arg("name", &actor.actor.name);
                state.add_text_arg("cur_hp", &actor.hp().to_string());
                state.add_text_arg("max_hp", &actor.stats.max_hp.to_string());
            },
            Kind::Prop(index) => {
                let area_state = GameState::area_state();
                let area_state = area_state.borrow();
                if !area_state.prop_index_valid(index) {
                    state.set_visible(false);
                    return false;
                }

                let prop = area_state.get_prop(index);

                if !prop.is_hover() && !prop.might_contain_items() {
                    state.add_text_arg("empty", "true");
                }
                state.add_text_arg("name", prop.name());

                if let Some(ref text) = prop.prop.status_text {
                    state.add_text_arg("status", text);
                }
            },
            Kind::Transition(ref name) => {
                state.add_text_arg("name", &name);
            },
        }

        true
    }
}

impl WidgetKind for AreaMouseover {
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

    fn on_mouse_move(&mut self, _widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        false
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        // prevent focus grab in root view where we compute mouse state each frame
        widget.borrow_mut().state.set_enabled(false);

        match self.kind {
            Kind::Entity(ref entity) => {
                entity.borrow_mut().actor.listeners.add(
                    ChangeListener::invalidate_layout(NAME, widget));
            },
            _ => (),
        }

        Vec::new()
    }

    fn layout(&mut self, widget: &mut Widget) {
        if !self.set_text_args(&mut widget.state) {
            widget.mark_for_removal();
        }

        self.text_area.borrow_mut().layout(widget);
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.text_area.borrow_mut().draw(renderer, pixel_size, widget, millis);
    }
}
