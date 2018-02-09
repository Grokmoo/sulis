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

use sulis_module::item::Slot;
use sulis_state::{EntityState, ChangeListener, GameState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};

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

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn layout(&mut self, widget: &mut Widget) {
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

        let list_content = Widget::empty("items_list");
        for (index, item) in actor.inventory().items.iter().enumerate() {
            if actor.inventory().is_equipped(index) {
                continue;
            }

            let button = Widget::with_theme(Button::empty(), "item_button");
            button.borrow_mut().state.add_text_arg("icon", &item.item.icon.id());

            match item.item.equippable {
                Some(_) => {
                    button.borrow_mut().state.add_callback(Callback::with(Box::new(move || {
                        let pc = GameState::pc();
                        let mut pc = pc.borrow_mut();

                        pc.actor.equip(index);
                    })));
                },
                None => (),
            };

            Widget::add_child_to(&list_content, button);
        }

        let equipped_area = Widget::empty("equipped_area");
        for slot in Slot::iter() {
            let theme_id = format!("{:?}_button", slot).to_lowercase();
            let button = Widget::with_theme(Button::empty(), &theme_id);

            button.borrow_mut().state.add_callback(Callback::with(Box::new(move || {
                let pc = GameState::pc();
                let mut pc = pc.borrow_mut();

                pc.actor.unequip(*slot);
            })));
            match actor.inventory().get(*slot) {
                None => button.borrow_mut().state.disable(),
                Some(ref item_state) => {
                    button.borrow_mut().state.add_text_arg("icon", &item_state.item.icon.id());
                }
            }

            Widget::add_child_to(&equipped_area, button);
        }

        vec![title, close, equipped_area, list_content]
    }
}
