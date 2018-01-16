use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::collections::vec_deque::Iter;

use state::{AreaState, EntityState};

/// `TurnTimer` maintains a list of all entities in a given `AreaState`.  The
/// list proceed in initiative order, with the front of the list always containing
/// the currently active entity.  Once an entity's turn is up, it is moved to the
/// back of the list.  Internally, this is accomplished using a `VecDeque`
pub struct TurnTimer {
    entities: VecDeque<Rc<RefCell<EntityState>>>,
}

impl Default for TurnTimer {
    fn default() -> TurnTimer {
        TurnTimer {
            entities: VecDeque::new(),
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

        TurnTimer {
            entities,
        }
    }

    pub fn remove(&mut self, entity: &Rc<RefCell<EntityState>>) {
        self.entities.retain(|other| *entity.borrow() != *other.borrow());
    }

    pub fn current(&self) -> Option<&Rc<RefCell<EntityState>>> {
        self.entities.front()
    }

    pub fn next(&mut self) {
        if self.entities.front().is_none() { return; }

        let front = self.entities.pop_front().unwrap();
        self.entities.push_back(front);
    }

    pub fn iter(&self) -> Iter<Rc<RefCell<EntityState>>> {
        self.entities.iter()
    }
}
