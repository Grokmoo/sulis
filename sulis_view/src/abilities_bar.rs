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

use sulis_module::Ability;
use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_core::io::event;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_widgets::{Label};

use BasicMouseover;

pub const NAME: &str = "abilities_bar";

pub struct AbilitiesBar {
    entity: Rc<RefCell<EntityState>>,
}

impl AbilitiesBar {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilitiesBar>> {
        Rc::new(RefCell::new(AbilitiesBar {
            entity,
        }))
    }
}

impl WidgetKind for AbilitiesBar {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        {
            let mut entity = self.entity.borrow_mut();
            entity.actor.listeners.add(ChangeListener::invalidate(NAME, widget));
        }

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |entity| {
            let entity = match entity {
                None => return,
                Some(entity) => entity,
            };
            let bar = Widget::downcast_kind_mut::<AbilitiesBar>(&widget_ref);
            bar.entity = Rc::clone(entity);
            widget_ref.borrow_mut().invalidate_children();
        })));

        let mut children = Vec::new();
        let abilities = self.entity.borrow().actor.actor.abilities.clone();
        for ability in abilities.iter() {
            if self.entity.borrow_mut().actor.ability_state(&ability.id).is_none() { continue; }

            let button = Widget::with_defaults(AbilityButton::new(&ability, &self.entity));
            children.push(button);
        }

        children
    }
}

struct AbilityButton {
    entity: Rc<RefCell<EntityState>>,
    ability: Rc<Ability>,
}

impl AbilityButton {
    fn new(ability: &Rc<Ability>, entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilityButton>> {
        Rc::new(RefCell::new(AbilityButton {
            ability: Rc::clone(ability),
            entity: Rc::clone(entity),
        }))
    }
}

impl WidgetKind for AbilityButton {
    widget_kind!("ability_button");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        widget.state.set_enabled(self.entity.borrow().actor.can_toggle(&self.ability.id));

        if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&self.ability.id) {
            let rounds = state.remaining_duration_rounds();

            widget.children[1].borrow_mut().state.clear_text_args();
            if rounds == 0 {
            } else if rounds < 100_000 {
                widget.children[1].borrow_mut().state.add_text_arg("duration", &rounds.to_string());
            } else {
                widget.children[1].borrow_mut().state.add_text_arg("duration", "Active");
            }
        }
    }

    fn on_remove(&mut self) {
        if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&self.ability.id) {
            state.listeners.remove(&self.ability.id);
        }
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&self.ability.id) {
            state.listeners.add(ChangeListener::invalidate_layout(&self.ability.id, widget));
        }

        let duration_label = Widget::with_theme(Label::empty(), "duration_label");
        duration_label.borrow_mut().state.set_enabled(false);
        let icon = Widget::empty("icon");
        icon.borrow_mut().state.add_text_arg("icon", &self.ability.icon.id());
        icon.borrow_mut().state.set_enabled(false);

        vec![icon, duration_label]
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _: f32, _: f32) -> bool {
        let x = widget.borrow().state.inner_left();
        let y = widget.borrow().state.inner_bottom();
        Widget::set_mouse_over(widget, BasicMouseover::new(&self.ability.name),
            x, y);
        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        if self.entity.borrow().actor.can_activate(&self.ability.id) {
            info!("Activate {}", self.ability.id);
            GameState::execute_ability_on_activate(&self.entity, &self.ability);
        } else if self.entity.borrow().actor.can_toggle(&self.ability.id) {
            self.entity.borrow_mut().actor.deactivate_ability_state(&self.ability.id);
        }

        true
    }
}
