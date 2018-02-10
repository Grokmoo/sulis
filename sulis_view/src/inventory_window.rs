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

use sulis_module::Equippable;
use sulis_module::item::Slot;
use sulis_state::{EntityState, ChangeListener, GameState};
use sulis_core::io::event;
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_widgets::{Button, Label, TextArea};

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

            let button = Widget::with_defaults(ItemButton::new(Some(index)));
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
            let button = Widget::with_theme(ItemButton::new(actor.inventory().get_index(*slot)), &theme_id);

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

struct ItemButton {
    button: Rc<RefCell<Button>>,
    item_window: Option<Rc<RefCell<Widget>>>,
    item_index: Option<usize>,
}

const ITEM_BUTTON_NAME: &str = "item_button";

impl ItemButton {
    pub fn new(index: Option<usize>) -> Rc<RefCell<ItemButton>> {
        Rc::new(RefCell::new(ItemButton {
            button: Button::empty(),
            item_window: None,
            item_index: index,
        }))
    }

    fn remove_item_window(&mut self) {
        if self.item_window.is_some() {
            self.item_window.as_ref().unwrap().borrow_mut().mark_for_removal();
            self.item_window = None;
        }
    }
}

impl WidgetKind for ItemButton {
    fn get_name(&self) -> &str { ITEM_BUTTON_NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);

        if self.item_index.is_some() && self.item_window.is_none() {
            let pc = GameState::pc();
            let pc = pc.borrow_mut();
            let item_state = &pc.actor.inventory().items[self.item_index.unwrap()];

            let item_window = Widget::with_theme(TextArea::empty(), "item_window");
            {
                let mut item_window = item_window.borrow_mut();
                item_window.state.disable();
                item_window.state.set_position(widget.borrow().state.inner_right(),
                (widget.borrow().state.inner_top() + widget.borrow().state.inner_bottom()) / 2);

                item_window.state.add_text_arg("name", &item_state.item.name);

                match item_state.item.equippable {
                    None => (),
                    Some(equippable) => {
                        add_equippable_text_args(&equippable, &mut item_window.state);
                    },
                }
            }
            let root = Widget::get_root(widget);
            Widget::add_child_to(&root, Rc::clone(&item_window));
            self.item_window = Some(item_window);
        }
        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);

        self.remove_item_window();
        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.remove_item_window();
        self.button.borrow_mut().on_mouse_release(widget, kind)
    }
}

fn add_equippable_text_args(equippable: &Equippable, widget_state: &mut WidgetState) {
    if let Some(damage) = equippable.damage {
        widget_state.add_text_arg("min_damage", &damage.min.to_string());
        widget_state.add_text_arg("max_damage", &damage.max.to_string());
        widget_state.add_text_arg("damage_kind", &damage.kind.to_string());
    }
}
