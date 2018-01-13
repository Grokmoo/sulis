use module::Item;

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemState {
    pub item: Rc<Item>,
}

impl ItemState {
    pub fn new(item: Rc<Item>) -> ItemState {
        ItemState {
            item
        }
    }
}
