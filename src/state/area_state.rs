use resource::Area;
use resource::Actor;

use state::ActorState;
use state::Location;

use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState<'a> {
    pub area: &'a Area,
    pub actors: Vec<Rc<RefCell<ActorState<'a>>>>,

    display: Vec<Vec<char>>,
}

impl<'a> PartialEq for AreaState<'a> {
    fn eq(&self, other: &AreaState<'a>) -> bool {
        self.area == other.area 
    }
}

impl<'a> AreaState<'a> {
    pub fn new(area: &'a Area) -> AreaState<'a> {
        let mut display = vec![vec![' ';area.width as usize];area.height as usize];
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = display.get_mut(y).unwrap().get_mut(x).unwrap();
                *cell = area.terrain_display_at(x, y);
            }
        }

        AreaState {
            area,
            actors: Vec::new(),
            display
        }
    }

    pub fn get_display(&self, x: usize, y: usize) -> char {
        *self.display.get(y).unwrap().get(x).unwrap()
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
        match self.area.terrain_at(x, y) {
            Some(ref tile) => {
                if !tile.passable { return false; }
            }, None => return false
        };
       
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
                let cell = self.display.get_mut(y).unwrap().get_mut(x).unwrap();
                *cell = actor.display;
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
                let cell = self.display.get_mut(y).unwrap().get_mut(x).unwrap();
                *cell = self.area.terrain_display_at(x, y);
            }
        }

        for y in new_y..(new_y + size) {
            for x in new_x..(new_x + size) {
                let cell = self.display.get_mut(y).unwrap().get_mut(x).unwrap();
                *cell = actor.actor.display;
            }
        }
    }
}
