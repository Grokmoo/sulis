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
use sulis_rules::QuickSlot;
use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_widgets::{Label, Button};
use {item_button::use_item_cb, ItemButton};

pub const NAME: &str = "quick_item_bar";

pub struct QuickItemBar {
    entity: Rc<RefCell<EntityState>>,
}

impl QuickItemBar {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<QuickItemBar>> {
        Rc::new(RefCell::new(QuickItemBar {
            entity: Rc::clone(entity),
        }))
    }
}

impl WidgetKind for QuickItemBar {
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
            let bar = Widget::downcast_kind_mut::<QuickItemBar>(&widget_ref);
            bar.entity = Rc::clone(entity);
            widget_ref.borrow_mut().invalidate_children();
        })));

        let title = Widget::with_theme(Label::empty(), "title");

        let swap_weapons = Widget::with_theme(Button::empty(), "swap_weapons");
        swap_weapons.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let bar = Widget::downcast_kind_mut::<QuickItemBar>(&parent);
            bar.entity.borrow_mut().actor.swap_weapon_set();
        })));

        let usable1 = create_button(&self.entity, QuickSlot::Usable1, "usable1");
        let usable2 = create_button(&self.entity, QuickSlot::Usable2, "usable2");
        let usable3 = create_button(&self.entity, QuickSlot::Usable3, "usable3");
        let usable4 = create_button(&self.entity, QuickSlot::Usable4, "usable4");

        vec![title, swap_weapons, usable1, usable2, usable3, usable4]
    }
}

fn create_button(entity: &Rc<RefCell<EntityState>>, slot: QuickSlot,
                 theme_id: &str) -> Rc<RefCell<Widget>> {
    let actor = &entity.borrow().actor;
    match actor.inventory().get_quick_index(slot) {
        None => {
            let button = Widget::empty(theme_id);
            button.borrow_mut().state.set_enabled(false);
            button
        }, Some(index) => {
            let (qty, item_state) = actor.inventory().items.get(index).unwrap();
            let button = ItemButton::inventory(entity, item_state.item.icon.id(),
                *qty, index);
            button.borrow_mut().add_action("Use", use_item_cb(entity, index));
            Widget::with_theme(button, theme_id)
        }
    }
}
