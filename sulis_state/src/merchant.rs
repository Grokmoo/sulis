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

use std::io::Error;
use std::rc::Rc;

use sulis_core::util::invalid_data_error;
use sulis_module::{LootList, Module};

use crate::{ChangeListenerList, ItemList, ItemState, save_state::MerchantSaveState};

pub struct Merchant {
    pub id: String,
    pub buy_frac: f32,
    pub sell_frac: f32,
    pub listeners: ChangeListenerList<Merchant>,
    items: ItemList,
}

impl Merchant {
    pub fn load(save: MerchantSaveState) -> Result<Merchant, Error> {
        let mut items = ItemList::new();
        for item_save in save.items {
            let item = match Module::item(&item_save.item.id) {
                None => invalid_data_error(&format!("No item with ID '{}'", item_save.item.id)),
                Some(item) => Ok(item),
            }?;

            items.add_quantity(item_save.quantity, ItemState::new(item));
        }

        Ok(Merchant {
            id: save.id,
            buy_frac: save.buy_frac,
            sell_frac: save.sell_frac,
            listeners: ChangeListenerList::default(),
            items,
        })
    }

    pub fn new(id: &str, loot_list: &Rc<LootList>, buy_frac: f32, sell_frac: f32) -> Merchant {
        let mut items = ItemList::new();

        for (qty, item) in loot_list.generate() {
            let item_state = ItemState::new(item);
            items.add_quantity(qty, item_state);
        }

        Merchant {
            id: id.to_string(),
            buy_frac,
            sell_frac,
            items,
            listeners: ChangeListenerList::default(),
        }
    }

    pub fn get_buy_price(&self, item_state: &ItemState) -> i32 {
        ((item_state.item.value as f32) * self.buy_frac).ceil() as i32
    }

    pub fn get_sell_price(&self, item_state: &ItemState) -> i32 {
        ((item_state.item.value as f32) * self.sell_frac).floor() as i32
    }

    pub fn add(&mut self, item_state: ItemState) {
        self.items.add(item_state);

        self.listeners.notify(&self);
    }

    /// removes one copy of the item at the specified index
    pub fn remove(&mut self, index: usize) -> Option<ItemState> {
        let result = self.items.remove(index);

        if result.is_some() {
            self.listeners.notify(&self);
        }

        result
    }

    pub fn items(&self) -> &ItemList {
        &self.items
    }
}
