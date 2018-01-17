use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;
pub use std::collections::vec_deque::Iter;

use state::{AreaState, ChangeListenerList, EntityState};

/// `TurnTimer` maintains a list of all entities in a given `AreaState`.  The
/// list proceed in initiative order, with the front of the list always containing
/// the currently active entity.  Once an entity's turn is up, it is moved to the
/// back of the list.  Internally, this is accomplished using a `VecDeque`
pub struct TurnTimer {
    entities: VecDeque<Rc<RefCell<EntityState>>>,
    pub listeners: ChangeListenerList<TurnTimer>,
}

impl Default for TurnTimer {
    fn default() -> TurnTimer {
        TurnTimer {
            entities: VecDeque::new(),
            listeners: ChangeListenerList::default(),
        }
    }
}

impl TurnTimer {
    pub fn new(area_state: &AreaState) -> TurnTimer {
        let mut entities: Vec<(u32, Rc<RefCell<EntityState>>)> = Vec::new();

        for entity in area_state.entity_iter() {
            let initiative = entity.borrow().actor.stats.initiative;
            entities.push((initiative, entity));
        }

        // sort by initiative
        entities.sort_by(|a, b| b.0.cmp(&a.0));

        let entities: VecDeque<Rc<RefCell<EntityState>>> = entities.into_iter()
            .map(|(_initiative, entity)| entity).collect();

        if let Some(entity) = entities.front() {
            entity.borrow_mut().actor.init_turn();
        }

        trace!("Got {} entities for turn timer", entities.len());
        TurnTimer {
            entities,
            ..Default::default()
        }
    }

    pub fn add(&mut self, entity: &Rc<RefCell<EntityState>>) {
        trace!("Added entity to turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entities.push_back(Rc::clone(entity));
        self.listeners.notify(&self);
    }

    pub fn remove(&mut self, entity: &Rc<RefCell<EntityState>>) {
        trace!("Removing entity from turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entities.retain(|other| *entity.borrow() != *other.borrow());
        self.listeners.notify(&self);
    }

    pub fn current(&self) -> Option<&Rc<RefCell<EntityState>>> {
        self.entities.front()
    }

    pub fn next(&mut self) {
        if self.entities.front().is_none() { return; }

        let front = self.entities.pop_front().unwrap();
        front.borrow_mut().actor.end_turn();
        self.entities.push_back(front);
        self.listeners.notify(&self);

        if let Some(current) = self.entities.front() {
            current.borrow_mut().actor.init_turn();
        }
    }

    pub fn iter(&self) -> Iter<Rc<RefCell<EntityState>>> {
        self.entities.iter()
    }
}
