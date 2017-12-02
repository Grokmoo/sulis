use resource::Area;
use resource::Actor;

use state::EntityState;
use state::Location;

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState<'a> {
    pub area: &'a Area,
    pub entities: Vec<Rc<RefCell<EntityState<'a>>>>,

    display: Vec<char>,
}

impl<'a> PartialEq for AreaState<'a> {
    fn eq(&self, other: &AreaState<'a>) -> bool {
        self.area == other.area 
    }
}

impl<'a> AreaState<'a> {
    pub fn new(area: &'a Area) -> AreaState<'a> {
        let mut display = vec![' ';area.width * area.height];
        for (index, element) in display.iter_mut().enumerate() {
            *element = area.terrain.display(index);
        }

        AreaState {
            area,
            entities: Vec::new(),
            display
        }
    }

    pub fn get_display(&self, x: usize, y: usize) -> char {
        *self.display.get(x + y * self.area.width).unwrap()
    }

    pub fn is_passable(&self, requester: Ref<EntityState<'a>>,
                       new_x: usize, new_y: usize) -> bool {
        requester.points(new_x, new_y).all(|p| self.point_passable(&requester, p.x, p.y))
    }

    fn point_passable(&self, requester: &Ref<EntityState<'a>>, x: usize, y: usize) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        if !self.area.terrain.at(x, y).passable { return false; }
       
        for entity in self.entities.iter() {
            let entity = entity.borrow();

            if *entity == **requester { continue; }
            if entity.location.equals(x, y) { return false; }
        }

        true
    }

    pub(in state) fn add_actor(&mut self, actor: Rc<Actor>,
                     location: Location<'a>) -> bool {

        let entity = EntityState::new(actor, location);

        let x = entity.location.x;
        let y = entity.location.y;

        let entity = Rc::new(RefCell::new(entity));

        if !self.is_passable(entity.borrow(), x, y) {
            return false;
        }

        entity.borrow().points(x, y).
            for_each(|p| self.update_display(p.x, p.y, entity.borrow().display()));

        self.entities.push(entity);

        true
    }

    pub(in state) fn update_entity_display(&mut self, entity: &EntityState<'a>, new_x: usize, new_y: usize) {
        let cur_x = entity.location.x;
        let cur_y = entity.location.y;

        entity.points(cur_x, cur_y).
            for_each(|p| self.update_display(p.x, p.y, self.area.terrain.display_at(p.x, p.y)));

        entity.points(new_x, new_y).
            for_each(|p| self.update_display(p.x, p.y, entity.display()));
    }

    fn update_display(&mut self, x: usize, y: usize, c: char) {
        *self.display.get_mut(x + y * self.area.width).unwrap() = c;
    }
}
