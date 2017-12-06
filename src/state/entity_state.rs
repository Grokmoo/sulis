use resource::Actor;
use resource::Size;
use resource::SizeIterator;
use state::Location;
use state::ActorState;

use std::rc::Rc;

pub struct EntityState<'a> {
    pub actor: ActorState,
    pub(in state) location: Location<'a>,
    size: Rc<Size>,
}

impl<'a> PartialEq for EntityState<'a> {
    fn eq(&self, other: &EntityState) -> bool {
        self.actor == other.actor
    }
}

impl<'a> EntityState<'a> {
    pub(in state) fn new(actor: Rc<Actor>, location: Location) -> EntityState {
        let size = Rc::clone(&actor.default_size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state, location, size,
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize) -> bool {
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.size() - 1, y + self.size() - 1) {
            return false;
        }

        self.location.area_state.borrow_mut().update_entity_display(&self, x, y);
        self.location.move_to(x, y);
        true
    }

    pub(in state) fn display(&self) -> char {
        self.actor.actor.display
    }

    pub fn size(&self) -> usize {
        self.size.size
    }

    pub fn relative_points(&self) -> SizeIterator {
        self.size.relative_points()
    }

    pub fn points(&self, x: usize, y: usize) -> SizeIterator {
        self.size.points(x, y)
    }
}

