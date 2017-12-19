use resource::Actor;
use state::Inventory;

use std::rc::Rc;

#[derive(Clone)]
pub struct ActorState {
    pub actor: Rc<Actor>,
    pub inventory: Inventory,
}

impl PartialEq for ActorState {
    fn eq(&self, other: &ActorState) -> bool {
        Rc::ptr_eq(&self.actor, &other.actor)
    }
}

impl ActorState {
    pub fn new(actor: Rc<Actor>) -> ActorState {
        trace!("Creating new actor state for {}", actor.id);
        let inventory = Inventory::new(&actor);
        ActorState {
            actor,
            inventory,
        }
    }
}
