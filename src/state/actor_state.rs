use resource::Actor;

use std::rc::Rc;

#[derive(PartialEq)]
pub struct ActorState {
    pub actor: Rc<Actor>,
}

impl ActorState {
    pub fn new(actor: Rc<Actor>) -> ActorState {
        ActorState {
            actor
        }
    } 
}
