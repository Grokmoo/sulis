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
use std::cell::{Cell, RefCell};

use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};

use crate::{ItemListPane, item_list_pane::Filter};

pub const NAME: &str = "merchant_window";

pub struct MerchantWindow {
    merchant_id: String,
    player: Rc<RefCell<EntityState>>,
    filter: Rc<Cell<Filter>>,
}

impl MerchantWindow {
    pub fn new(merchant_id: &str, player: Rc<RefCell<EntityState>>) -> Rc<RefCell<MerchantWindow>> {
        Rc::new(RefCell::new(MerchantWindow {
            merchant_id: merchant_id.to_string(),
            player,
            filter: Rc::new(Cell::new(Filter::All)),
        }))
    }

    pub fn merchant_id(&self) -> &str { &self.merchant_id }

    pub fn player(&self) -> &Rc<RefCell<EntityState>> { &self.player }
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
        {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();

            let mut merchant = area_state.get_merchant_mut(&self.merchant_id);
            let merchant = match merchant {
                None => return Vec::new(),
                Some(ref mut merchant) => merchant,
            };

            merchant.listeners.add(ChangeListener::invalidate(NAME, widget));
        }

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let item_list_pane = Widget::with_defaults(
            ItemListPane::new_merchant(&self.player, self.merchant_id.to_string(), &self.filter));

        vec![title, close, item_list_pane]
    }
}
