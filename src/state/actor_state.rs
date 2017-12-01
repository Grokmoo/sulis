use resource::Actor;
use state::Location;

#[derive(PartialEq)]
pub struct ActorState<'a> {
    pub actor: &'a Actor,
    pub(in state) location: Location<'a>,
}

impl<'a> ActorState<'a> {
    pub(in state) fn move_to(&mut self, x: usize, y: usize) -> bool {
        if !self.location.coords_valid(x, y) { return false; }
        if !self.location.coords_valid(x + self.actor.size - 1,y + self.actor.size - 1) {
            return false;
        }

        self.location.area_state.borrow_mut().update_actor_display(&self, x, y);
        self.location.move_to(x, y);
        true
    }
}
