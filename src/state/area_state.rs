use resource::Area;
use resource::Actor;

use state::ActorState;
use state::Location;

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState<'a> {
    pub area: &'a Area,
    pub actors: Vec<Rc<RefCell<ActorState<'a>>>>,

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
            actors: Vec::new(),
            display
        }
    }

    pub fn get_display(&self, x: usize, y: usize) -> char {
        *self.display.get(x + y * self.area.width).unwrap()
    }

    pub fn is_passable(&self, requester: Ref<ActorState<'a>>,
                       new_x: usize, new_y: usize) -> bool {
        let size = requester.actor.size;
        
        for y in new_y..(new_y + size) {
            for x in new_x..(new_x + size) {
                if !self.point_passable(&requester, x, y) { return false; }
            }
        }
    
        true
    }

    fn point_passable(&self, requester: &Ref<ActorState<'a>>, x: usize, y: usize) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        if !self.area.terrain.at(x, y).passable { return false; }
       
        for actor in self.actors.iter() {
            let actor = actor.borrow();

            if *actor == **requester { continue; }
            if actor.location.equals(x, y) { return false; }
        }

        true
    }

    pub(in state) fn add_actor(&mut self, actor: &'a Actor,
                     location: Location<'a>) -> bool {
        let actor_state = ActorState {
            actor: actor,
            location: location,
        };

        let x = actor_state.location.x;
        let y = actor_state.location.y;

        let actor_state = Rc::new(RefCell::new(actor_state));

        if !self.is_passable(actor_state.borrow(), x, y) {
            return false;
        }

        for y in y..(y + actor.size) {
            for x in x..(x + actor.size) {
                self.update_display(x, y, actor.display);
            }
        }

        self.actors.push(actor_state);

        true
    }

    pub(in state) fn update_actor_display(&mut self, actor: &ActorState<'a>, new_x: usize, new_y: usize) {
        let cur_x = actor.location.x;
        let cur_y = actor.location.y;
        let size = actor.actor.size;

        for y in cur_y..(cur_y + size) {
            for x in cur_x..(cur_x + size) {
                self.update_display(x, y, self.area.terrain.display_at(x, y));
            }
        }

        for y in new_y..(new_y + size) {
            for x in new_x..(new_x + size) {
                self.update_display(x, y, actor.actor.display);
            }
        }
    }

    fn update_display(&mut self, x: usize, y: usize, c: char) {
        *self.display.get_mut(x + y * self.area.width).unwrap() = c;
    }
}
