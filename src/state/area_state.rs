use module::{Actor, Area, Module};
use module::area::Transition;
use state::{ChangeListenerList, EntityState, Location, TurnTimer};

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState {
    pub area: Rc<Area>,
    pub listeners: ChangeListenerList<AreaState>,
    pub turn_timer: TurnTimer,
    entities: Vec<Option<Rc<RefCell<EntityState>>>>,

    entity_grid: Vec<Option<usize>>,
    transition_grid: Vec<Option<usize>>,
    display: Vec<char>,
}

impl PartialEq for AreaState {
    fn eq(&self, other: &AreaState) -> bool {
        self.area == other.area
    }
}

impl AreaState {
    pub fn new(area: Rc<Area>) -> AreaState {
        let mut display = vec![' ';(area.width * area.height) as usize];
        for (index, element) in display.iter_mut().enumerate() {
            *element = area.terrain.display(index);
        }

        let entity_grid = vec![None;(area.width * area.height) as usize];
        let transition_grid = vec![None;(area.width * area.height) as usize];

        AreaState {
            area,
            entities: Vec::new(),
            turn_timer: TurnTimer::default(),
            transition_grid,
            display,
            entity_grid,
            listeners: ChangeListenerList::default(),
        }
    }

    /// Adds entities defined in the area definition to this area state
    pub fn populate(&mut self) {
        let area = Rc::clone(&self.area);
        for actor_data in area.actors.iter() {
            let actor = match Module::actor(&actor_data.id) {
                None => {
                    warn!("No actor with id '{}' found when initializing area '{}'",
                              actor_data.id, self.area.id);
                    continue;
                },
                Some(actor_data) => actor_data,
            };

            let location = Location::from_point(&actor_data.location, &self.area);
            debug!("Adding actor '{}' at '{:?}'", actor.id, location);
            self.add_actor(actor, location, false);
        }

        let turn_timer = TurnTimer::new(&self);
        self.turn_timer = turn_timer;
        trace!("Set up turn timer for area.");

        for (index, transition) in self.area.transitions.iter().enumerate() {
            debug!("Adding transition '{}' at '{:?}'", index, transition.from);
            for y in 0..transition.size.height {
                for x in 0..transition.size.width {
                    self.transition_grid[(transition.from.x + x +
                        (transition.from.y + y) * self.area.width) as usize] = Some(index);
                }
            }
        }
    }

    pub fn get_display(&self, x: i32, y: i32) -> char {
        *self.display.get((x + y * self.area.width) as usize).unwrap()
    }

    pub fn is_passable(&self, requester: &Ref<EntityState>,
                       new_x: i32, new_y: i32) -> bool {
        if !self.area.coords_valid(new_x, new_y) { return false; }

        if !self.area.get_path_grid(requester.size()).is_passable(new_x, new_y) {
            return false;
        }

        requester.points(new_x, new_y)
            .all(|p| self.point_entities_passable(&requester, p.x, p.y))
    }

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<Rc<RefCell<EntityState>>> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.entity_grid.get((x + y * self.area.width) as usize).unwrap() {
            &None => return None,
            &Some(index) => index,
        };

        Some(self.get_entity(index))
    }

    pub fn get_transition_at(&self, x: i32, y: i32) -> Option<&Transition> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.transition_grid[(x + y * self.area.width) as usize] {
            None => return None,
            Some(index) => index,
        };

        self.area.transitions.get(index)
    }

    fn point_entities_passable(&self, requester: &Ref<EntityState>,
                               x: i32, y: i32) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        let grid_index = self.entity_grid.get((x + y * self.area.width) as usize);

        match grid_index {
            None => false, // grid position is invalid
            Some(&None) => true, // grid position is valid, location is empty
            Some(&Some(index)) => (index == requester.index),
        }
    }

    pub(in state) fn add_actor(&mut self, actor: Rc<Actor>,
                     location: Location, is_pc: bool) -> bool {
        let entity = EntityState::new(actor, location.clone(), 0, is_pc);
        let entity = Rc::new(RefCell::new(entity));
        self.add_entity(entity, location)
    }

    pub(in state) fn add_entity(&mut self, entity: Rc<RefCell<EntityState>>,
                                location: Location) -> bool {
        let x = location.x;
        let y = location.y;

        if !self.is_passable(&entity.borrow(), x, y) {
            return false;
        }

        entity.borrow_mut().actor.compute_stats();
        entity.borrow_mut().actor.init();

        let new_index = self.find_index_to_add();
        entity.borrow_mut().index = new_index;
        entity.borrow_mut().location = location;

        for p in entity.borrow().points(x, y) {
            self.update_display(p.x, p.y, entity.borrow().display());
            self.update_entity_grid(p.x, p.y, Some(new_index));
        }

        self.turn_timer.add(&entity);
        self.entities[new_index] = Some(entity);

        self.listeners.notify(&self);
        true
    }

    pub(in state) fn update_entity_position(&mut self, entity: &EntityState,
                                           new_x: i32, new_y: i32) {
        self.clear_entity_points(entity);

        for p in entity.points(new_x, new_y) {
            self.update_display(p.x, p.y, entity.display());
            self.update_entity_grid(p.x, p.y, Some(entity.index));
        }
    }

    fn clear_entity_points(&mut self, entity: &EntityState) {
        let cur_x = entity.location.x;
        let cur_y = entity.location.y;

        for p in entity.points(cur_x, cur_y) {
            let c = self.area.terrain.display_at(p.x, p.y);
            self.update_display(p.x, p.y, c);
            self.update_entity_grid(p.x, p.y, None);
        }
    }

    fn update_entity_grid(&mut self, x: i32, y: i32, index: Option<usize>) {
        *self.entity_grid.get_mut((x + y * self.area.width) as usize).unwrap() = index;
    }

    fn update_display(&mut self, x: i32, y: i32, c: char) {
        *self.display.get_mut((x + y * self.area.width) as usize).unwrap() = c;
    }

    pub fn get_last_entity(&self) -> Option<&Rc<RefCell<EntityState>>> {
        for item in self.entities.iter().rev() {
            if let &Some(ref entity) = item {
                return Some(entity);
            }
        }

        None
    }

    pub fn entity_iter(&self) -> EntityIterator {
        EntityIterator { area_state: &self, index: 0 }
    }

    fn get_entity(&self, index: usize) -> Rc<RefCell<EntityState>> {
        let entity = &self.entities[index];

        Rc::clone(&entity.as_ref().unwrap())
    }

    pub (in state) fn update(&mut self) -> Option<&Rc<RefCell<EntityState>>> {
        // removal does not shuffle the vector around, so we can safely just iterate
        let mut notify = false;
        let len = self.entities.len();
        for index in 0..len {
            let entity = match &self.entities[index].as_ref() {
                &None => continue,
                &Some(entity) => Rc::clone(entity),
            };
            if !entity.borrow().is_marked_for_removal() { continue; }

            self.remove_entity_at_index(&entity, index);
            notify = true;
        }

        if notify {
            self.listeners.notify(&self);
        }

        self.turn_timer.current()
    }

    pub(in state) fn remove_entity(&mut self, entity: &Rc<RefCell<EntityState>>) {
        if !entity.borrow().location.is_in(&self) {
            warn!("Unable to remove entity '{}' from area '{}' as it is not in the area.",
                  entity.borrow().actor.actor.id, self.area.id);
        }

        let index = entity.borrow().index;
        self.remove_entity_at_index(entity, index);
        self.listeners.notify(&self);
    }

    fn remove_entity_at_index(&mut self, entity: &Rc<RefCell<EntityState>>, index: usize) {
        trace!("Removing entity '{}' with index '{}'", entity.borrow().actor.actor.name, index);
        self.clear_entity_points(&*entity.borrow());
        self.entities[index] = None;
        self.turn_timer.remove(entity);
    }

    fn find_index_to_add(&mut self) -> usize {
        for (index, item) in self.entities.iter().enumerate() {
            if item.is_none() {
                return index;
            }
        }

        self.entities.push(None);
        self.entities.len() - 1
    }
}

pub struct EntityIterator<'a> {
    area_state: &'a AreaState,
    index: usize,
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<Rc<RefCell<EntityState>>> {
        loop {
            let next = self.area_state.entities.get(self.index);

            self.index += 1;

            match next {
                None => return None,
                Some(e) => match e {
                    &None => continue,
                    &Some(ref entity) => return Some(Rc::clone(entity))
                }
            }
        }
    }
}
