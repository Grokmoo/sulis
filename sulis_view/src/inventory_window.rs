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

use std::rc::Rc;
use std::cell::RefCell;

use sulis_state::{EntityState, ChangeListener, GameState};
use sulis_core::ui::{animation_state, AnimationState, Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, list_box, ListBox};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    entity: Rc<RefCell<EntityState>>,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<InventoryWindow>> {
        Rc::new(RefCell::new(InventoryWindow {
            entity: Rc::clone(entity)
        }))
    }
}

impl WidgetKind for InventoryWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed inventory window.");
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let ref actor = self.entity.borrow().actor;

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, item) in actor.inventory().items.iter().enumerate() {
            let cb = match item.item.equippable {
                Some(equippable) => {
                    let slot = equippable.slot;
                    Some(Callback::with(Box::new(move || {
                        let pc = GameState::pc();
                        let mut pc = pc.borrow_mut();

                        if pc.actor.inventory().is_equipped(index) {
                            pc.actor.unequip(slot);
                        } else {
                            pc.actor.equip(index);
                        }
                    })))
                },
                None => None,
            };

            let entry = if actor.inventory().is_equipped(index) {
                list_box::Entry::with_state(item.item.name.to_string(), cb,
                    AnimationState::with(animation_state::Kind::Active))
            } else {
                list_box::Entry::new(item.item.name.to_string(), cb)
            };

            entries.push(entry);
        }

        let list = Widget::with_theme(ListBox::new(entries), "inventory");

        vec![title, close, list]
    }
}
