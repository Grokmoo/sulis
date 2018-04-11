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

use sulis_state::{ChangeListener, GameState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use inventory_window::ItemButton;

pub const NAME: &str = "merchant_window";

pub struct MerchantWindow {
    merchant_id: String,
}

impl MerchantWindow {
    pub fn new(merchant_id: &str) -> Rc<RefCell<MerchantWindow>> {
        Rc::new(RefCell::new(MerchantWindow {
            merchant_id: merchant_id.to_string(),
        }))
    }
}

impl WidgetKind for MerchantWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        if let Some(ref mut merchant) = area_state.get_merchant_mut(&self.merchant_id) {
            merchant.listeners.remove(NAME);
        }
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        let mut merchant = area_state.get_merchant_mut(&self.merchant_id);
        let merchant = match merchant {
            None => return Vec::new(),
            Some(ref mut merchant) => merchant,
        };

        merchant.listeners.add(ChangeListener::invalidate_layout(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let list_content = Widget::empty("items_list");
        for (index, &(qty, ref item)) in merchant.items().iter().enumerate() {
            let item_button = ItemButton::new(Some(item.item.icon.id()), qty,
                Some(index), None, Some(self.merchant_id.to_string()));
            let button = Widget::with_defaults(item_button);

            // button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            //     let pc = GameState::pc();
            //     let mut pc = pc.borrow_mut();
            //     pc.actor.take(prop_index, index);
            // })));
            Widget::add_child_to(&list_content, button);
        }

        vec![title, close, list_content]
    }
}
