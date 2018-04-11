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

use sulis_module::{LootList};

use {ChangeListenerList, ItemList, ItemState};

pub struct Merchant {
    pub id: String,
    pub listeners: ChangeListenerList<Merchant>,
    items: ItemList,
}

impl Merchant {
    pub fn new(id: &str, loot_list: &Rc<LootList>) -> Merchant {
        let mut items = ItemList::new();

        for item in loot_list.generate() {
            let item_state = ItemState::new(item);
            items.add(item_state);
        }

        Merchant {
            id: id.to_string(),
            items,
            listeners: ChangeListenerList::default(),
        }
    }

    pub fn items(&self) -> &ItemList {
        &self.items
    }
}
