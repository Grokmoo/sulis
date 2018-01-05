use grt::resource::{Actor, EntitySize, EntitySizeIterator};
use state::{ActorState, GameState, Location};

use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq)]
pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<EntitySize>,
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

    pub fn relative_points(&self) -> EntitySizeIterator {
        self.size.relative_points()
    }

    pub fn points(&self, x: i32, y: i32) -> EntitySizeIterator {
        self.size.points(x, y)
    }

    /// Returns true if `e1` and `e2` refer to the same EntityState, false
    /// otherwise
    pub fn equals(e1: &Rc<RefCell<EntityState>>, e2: &Rc<RefCell<EntityState>>) -> bool {
        if e1.borrow().location != e2.borrow().location {
            return false;
        }

        e1.borrow().index == e2.borrow().index
    }

    /// Returns true if `e` refers to the current selected PC, false otherwise
    pub fn is_pc(e: &Rc<RefCell<EntityState>>) -> bool {
        EntityState::equals(e, &GameState::pc())
    }
}

