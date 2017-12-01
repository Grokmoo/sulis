use std::rc::Rc;
use std::cell::RefCell;

use state::AreaState;

pub struct Location<'a> {
    pub x: usize,
    pub y: usize,
    pub area_state: Rc<RefCell<AreaState<'a>>>
}

impl<'a> PartialEq for Location<'a> {
    fn eq(&self, other: &Location<'a>) -> bool {
        if self.x != other.x || self.y != other.y { return false; }

        if &self.area_state != &other.area_state { return false; }

        true
    }
}

impl<'a> Location<'a> {
    pub fn new(x: usize, y: usize, area_state: Rc<RefCell<AreaState<'a>>>) -> Location<'a> {
        Location { x, y, area_state }
    }

    pub fn equals(&self, x: usize, y: usize) -> bool {
        return self.x == x && self.y == y
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn coords_valid(&self, x: usize, y: usize) -> bool{
        self.area_state.borrow().area.coords_valid(x, y)
    }
}
