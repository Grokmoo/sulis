use grt::resource::{Actor, Size, SizeIterator};
use state::Location;
use state::ActorState;

use std::rc::Rc;

#[derive(PartialEq)]
pub struct EntityState {
    pub actor: ActorState,
    pub(in state) location: Location,
    size: Rc<Size>,
    pub index: usize, // index in vec of the owning area state
}

impl EntityState {
    pub(in state) fn new(actor: Rc<Actor>, location: Location, index: usize) -> EntityState {
        debug!("Creating new entity state for {}", actor.id);
        let size = Rc::clone(&actor.default_size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state,
            location,
            size,
            index
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32) -> bool {
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.size() - 1, y + self.size() - 1) {
            return false;
        }

        self.location.area_state.borrow_mut().update_entity_position(&self, x, y);
        self.location.move_to(x, y);
        true
    }

    pub(in state) fn display(&self) -> char {
        self.actor.actor.display
    }

    pub fn size(&self) -> i32 {
        self.size.size
    }

    pub fn relative_points(&self) -> SizeIterator {
        self.size.relative_points()
    }

    pub fn points(&self, x: i32, y: i32) -> SizeIterator {
        self.size.points(x, y)
    }
}

