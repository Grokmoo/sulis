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

use sulis_module::{Module, Item};

use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ItemState {
    pub item: Rc<Item>,
}

impl PartialEq for ItemState {
    fn eq(&self, other: &ItemState) -> bool {
        Rc::ptr_eq(&self.item, &other.item)
    }
}

impl ItemState {
    pub fn new(item: Rc<Item>) -> ItemState {
        ItemState {
            item
        }
    }

    pub fn from(id: &str) -> Option<ItemState> {
        match Module::item(id) {
            None => None,
            Some(item) => Some(ItemState::new(item)),
        }
    }
}
