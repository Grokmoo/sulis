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

use sulis_state::{ChangeListener, EntityState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button};

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
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        let mut entity = self.entity.borrow_mut();
        entity.actor.listeners.add(ChangeListener::invalidate(NAME, widget));

        let mut children = Vec::new();
        for ability in entity.actor.actor.abilities.iter() {
            if !ability.active { continue; }

            let button = Widget::with_theme(Button::empty(), "ability_button");
            button.borrow_mut().state.add_text_arg("icon", &ability.icon.id());
            // TODO activate callback

            children.push(button);
        }

        children
    }
}
