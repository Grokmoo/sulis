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
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind, RcRfc};
use sulis_core::widgets::Button;
use sulis_state::{ChangeListener, EntityState, GameState};

use crate::{item_list_pane::Filter, ItemListPane};

pub const NAME: &str = "merchant_window";

pub struct MerchantWindow {
    merchant_id: String,
    player: RcRfc<EntityState>,
    filter: Rc<Cell<Filter>>,
}

impl MerchantWindow {
    pub fn new(merchant_id: &str, player: RcRfc<EntityState>) -> RcRfc<MerchantWindow> {
        Rc::new(RefCell::new(MerchantWindow {
            merchant_id: merchant_id.to_string(),
            player,
            filter: Rc::new(Cell::new(Filter::All)),
        }))
    }

    pub fn merchant_id(&self) -> &str {
        &self.merchant_id
    }

    pub fn player(&self) -> &RcRfc<EntityState> {
        &self.player
    }
}

impl WidgetKind for MerchantWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self, _widget: &RcRfc<Widget>) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        if let Some(ref mut merchant) = area_state.get_merchant_mut(&self.merchant_id) {
            merchant.listeners.remove(NAME);
        }
    }

    fn on_add(&mut self, widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();

            let mut merchant = area_state.get_merchant_mut(&self.merchant_id);
            let merchant = match merchant {
                None => return Vec::new(),
                Some(ref mut merchant) => merchant,
            };

            merchant
                .listeners
                .add(ChangeListener::invalidate(NAME, widget));
        }

        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<MerchantWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let item_list_pane = Widget::with_defaults(ItemListPane::new_merchant(
            &self.player,
            self.merchant_id.to_string(),
            &self.filter,
        ));

        vec![close, item_list_pane]
    }
}
