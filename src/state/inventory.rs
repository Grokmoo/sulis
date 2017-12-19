use std::rc::Rc;

use resource::Actor;
use state::ItemState;

#[derive(Clone)]
pub struct Inventory {
    pub items: Vec<ItemState>,
}

impl Inventory {
    pub fn new(actor: &Rc<Actor>) -> Inventory {
        let mut items: Vec<ItemState> = Vec::new();

        for item in actor.items.iter() {
            items.push(ItemState::new(Rc::clone(item)));
        }

        trace!("Populated initial inventory with {} items", items.len());
        Inventory {
            items
        }
    }
}
