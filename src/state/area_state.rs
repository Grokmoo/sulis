use grt::resource::{Actor, Area, ResourceSet};

use state::EntityState;
use state::Location;

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState {
    pub area: Rc<Area>,
    pub entities: Vec<Rc<RefCell<EntityState>>>,

    entity_grid: Vec<Option<usize>>,
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

        AreaState {
            area,
            entities: Vec::new(),
            display,
            entity_grid,
        }
    }

    /// Adds entities defined in the area definition to this area state
    pub fn populate(area_state: &Rc<RefCell<AreaState>>) {
        let area = Rc::clone(&area_state.borrow().area);

        for actor_data in area.actors.iter() {
            let actor = match ResourceSet::get_actor(&actor_data.id) {
                None => {
                    warn!("No actor with id '{}' found when initializing area '{}'",
                              actor_data.id, area.id);
                    continue;
                },
                Some(actor_data) => actor_data,
            };

            let location = Location::from_point(&actor_data.location,
                                                Rc::clone(&area_state));
            debug!("Adding actor '{}' at '{:?}'", actor.id, location);
            area_state.borrow_mut().add_actor(actor, location);
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

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<&Rc<RefCell<EntityState>>> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.entity_grid.get((x + y * self.area.width) as usize).unwrap() {
            &None => return None,
            &Some(index) => index,
        };

        self.entities.get(index)
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
                     location: Location) -> bool {

        let new_index = self.entities.len();
        let entity = EntityState::new(actor, location, new_index);

        let x = entity.location.x;
        let y = entity.location.y;

        let entity = Rc::new(RefCell::new(entity));

        if !self.is_passable(&entity.borrow(), x, y) {
            return false;
        }

        for p in entity.borrow().points(x, y) {
            self.update_display(p.x, p.y, entity.borrow().display());
            self.update_entity_grid(p.x, p.y, Some(new_index));
        }

        self.entities.push(entity);

        true
    }

    pub(in state) fn update_entity_position(&mut self, entity: &EntityState,
                                           new_x: i32, new_y: i32) {
        let cur_x = entity.location.x;
        let cur_y = entity.location.y;

        for p in entity.points(cur_x, cur_y) {
            let c = self.area.terrain.display_at(p.x, p.y);
            self.update_display(p.x, p.y, c);
            self.update_entity_grid(p.x, p.y, None);
        }

        for p in entity.points(new_x, new_y) {
            self.update_display(p.x, p.y, entity.display());
            self.update_entity_grid(p.x, p.y, Some(entity.index));
        }
    }

    fn update_entity_grid(&mut self, x: i32, y: i32, index: Option<usize>) {
        *self.entity_grid.get_mut((x + y * self.area.width) as usize).unwrap() = index;
    }

    fn update_display(&mut self, x: i32, y: i32, c: char) {
        *self.display.get_mut((x + y * self.area.width) as usize).unwrap() = c;
    }
}
