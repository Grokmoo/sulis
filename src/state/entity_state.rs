use module::{Actor, EntitySize, EntitySizeIterator};
use module::area::Transition;
use state::{ActorState, GameState, Location};

use std::rc::Rc;
use std::cell::RefCell;

pub struct EntityState {
    pub actor: ActorState,
    pub location: Location,
    pub size: Rc<EntitySize>,
    pub index: usize, // index in vec of the owning area state

    marked_for_removal: bool,
}

impl PartialEq for EntityState {
    fn eq(&self, other: &EntityState) -> bool {
        self.location.area_id == other.location.area_id &&
            self.index == other.index
    }
}

impl EntityState {
    pub(in state) fn new(actor: Rc<Actor>, location: Location, index: usize) -> EntityState {
        debug!("Creating new entity state for {}", actor.id);
        let size = Rc::clone(&actor.race.size);
        let actor_state = ActorState::new(actor);
        EntityState {
            actor: actor_state,
            location,
            size,
            index,
            marked_for_removal: false,
        }
    }

    pub (in state) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    pub fn remove_hp(&mut self, hp: u32) {
        self.actor.remove_hp(hp);

        if self.actor.hp() <= 0 {
            debug!("Entity '{}' has zero hit points.  Marked to remove.", self.actor.actor.name);
            self.marked_for_removal = true;
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

    fn dist(&self, to_x: i32, to_y: i32, to_size: i32) -> f32 {
        let self_half_size = self.size() as f32 / 2.0;
        let other_half_size = to_size as f32 / 2.0;
        let from_x = self.location.x as f32 + self_half_size;
        let from_y = self.location.y as f32 + self_half_size;
        let to_x = to_x as f32 + other_half_size;
        let to_y = to_y as f32 + other_half_size;

        ((from_x - to_x) * (from_x - to_x) + (from_y - to_y) * (from_y - to_y)).sqrt()
            - self_half_size - other_half_size
    }

    pub fn dist_to_entity(&self, other: &Rc<RefCell<EntityState>>) -> f32 {
        let value = self.dist(other.borrow().location.x, other.borrow().location.y, other.borrow().size());

        trace!("Computed distance from '{}' at {:?} to '{}' at {:?} = {}", self.actor.actor.name,
               self.location, other.borrow().actor.actor.name, other.borrow().location, value);

        value
    }

    pub fn dist_to_transition(&self, other: &Transition) -> f32 {
        let value = self.dist(other.from.x, other.from.y, other.size.width);

        trace!("Computed distance from '{}' at {:?} to transition at {:?} = {}",
               self.actor.actor.name, self.location, other.from, value);

        value
    }

    pub(in state) fn display(&self) -> char {
        self.actor.actor.text_display
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

