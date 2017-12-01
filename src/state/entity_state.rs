use resource::Entity;
use state::Location;

#[derive(PartialEq)]
pub struct EntityState<'a> {
    pub entity: &'a Entity,
    pub(in state) location: Location<'a>,
}

impl<'a> EntityState<'a> {
    pub(in state) fn move_to(&mut self, x: usize, y: usize) -> bool {
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.entity.size - 1,y + self.entity.size - 1) {
            return false;
        }

        self.location.area_state.borrow_mut().update_entity_display(&self, x, y);
        self.location.move_to(x, y);
        true
    }
}
