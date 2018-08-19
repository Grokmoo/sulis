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
use sulis_core::io::{GraphicsRenderer};
use sulis_core::util::Point;
use sulis_widgets::{TextArea};
use sulis_state::{ChangeListener, EntityState, GameState};

const NAME: &'static str = "area_mouseover";

enum Kind {
    Entity(Rc<RefCell<EntityState>>),
    Prop(usize),
    Transition(String),
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

    fn set_text_args(&self, state: &mut WidgetState) {
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
                let prop = area_state.get_prop(index);

                if !prop.might_contain_items() {
                    state.add_text_arg("empty", "true");
                }
                state.add_text_arg("name", &prop.prop.name);
            },
            Kind::Transition(ref name) => {
                state.add_text_arg("name", &name);
            },
        }
    }
}

impl WidgetKind for AreaMouseover {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
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
        self.set_text_args(&mut widget.state);

        // double layout - first to get the position, then to actually do the layout
        self.text_area.borrow_mut().layout(widget);
        // widget.state.position.y -= widget.state.size.height;
        // widget.state.position.x -= widget.state.size.width / 2;
        // self.text_area.borrow_mut().layout(widget);
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.text_area.borrow_mut().draw(renderer, pixel_size, widget, millis);
    }
}
