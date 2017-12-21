use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};

use resource::Point;
use state::AreaState;

pub struct Location<'a> {
    pub x: i32,
    pub y: i32,
    pub area_state: Rc<RefCell<AreaState<'a>>>,
    pub area_id: String,
}

impl<'a> Debug for Location<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "{{ {},{} in {} }}", self.x, self.y, self.area_id)
    }
}

impl<'a> PartialEq for Location<'a> {
    fn eq(&self, other: &Location<'a>) -> bool {
        if self.x != other.x || self.y != other.y { return false; }

        if &self.area_state != &other.area_state { return false; }

        true
    }
}

impl<'a> Location<'a> {
    pub fn new(x: i32, y: i32, area_state: Rc<RefCell<AreaState<'a>>>) -> Location<'a> {
        let area_id = area_state.borrow().area.id.clone();
        Location { x, y, area_state, area_id }
    }

    pub fn from_point(p: &Point, area_state: Rc<RefCell<AreaState<'a>>>) -> Location<'a> {
        let area_id = area_state.borrow().area.id.clone();
        Location { x: p.x, y: p.y, area_state, area_id }
    }

    pub fn equals(&self, x: i32, y: i32) -> bool {
        return self.x == x && self.y == y
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn coords_valid(&self, x: i32, y: i32) -> bool{
        self.area_state.borrow().area.coords_valid(x, y)
    }
}
