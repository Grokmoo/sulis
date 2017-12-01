mod area_state;
pub use self::area_state::AreaState;

mod actor_state;
pub use self::actor_state::ActorState;

use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use resource::ResourceSet;
use resource::Actor;
use config::Config;

pub struct GameState<'a> {
    config: Config,
    area_state: Rc<RefCell<AreaState<'a>>>,
    pc: Rc<RefCell<ActorState<'a>>>,
}

impl<'a> GameState<'a> {
    pub fn new(config: Config, resources: &'a ResourceSet) -> Result<GameState<'a>, Error> {
        let game = &resources.game;

        let area = resources.areas.get(&game.starting_area);
        let area = match area {
            Some(a) => a,
            None => {
                eprintln!("Starting area '{}' not found", &game.starting_area);
                return Err(Error::new(ErrorKind::NotFound,
                                      "Unable to create starting area."));
            }
        };
        let area_state = Rc::new(RefCell::new(AreaState::new(area)));

        if game.starting_location.len() != 2 {
            eprintln!("Starting location must be an integer array of length 2.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }
        let x = game.starting_location.get(0).unwrap();
        let y = game.starting_location.get(1).unwrap();
        let location = Location::new(*x, *y, Rc::clone(&area_state));

        if !location.coords_valid(location.x, location.y) {
            eprintln!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }

        let pc = resources.actors.get(&game.pc);
        let pc = match pc {
            Some(a) => a,
            None => {
                eprintln!("Player character '{}' not found", &game.pc);
                return Err(Error::new(ErrorKind::NotFound,
                                      "Unable to create player character."));
            }
        };
        
        if !area_state.borrow_mut().add_actor(pc, location) {
            eprintln!("Player character starting location must be within area bounds and passable.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to add player character to starting area at starting location"));
        }
        
        let pc_state = Rc::clone(area_state.borrow().actors.last().unwrap());

        Ok(GameState {
            config: config,
            area_state: area_state,
            pc: pc_state,
        })
    }
    
    pub fn pc(&self) -> Ref<ActorState<'a>> {
        self.pc.borrow()
    }

    pub fn pc_mut(&mut self) -> RefMut<ActorState<'a>> {
        self.pc.borrow_mut()
    }

    pub fn area_state(&self) -> Ref<AreaState<'a>> {
        self.area_state.borrow()
    }

    pub fn area_state_mut(&mut self) -> RefMut<AreaState<'a>> {
        self.area_state.borrow_mut()
    }

    pub fn handle_input(&mut self, c: char) {
        let action = {
            let action = self.config.get_input_action(c);

            if let None = action { return; }

            *action.unwrap()
        };

        use config::InputAction::*;
        match action {
            MoveUp => self.pc_move_by(0, -1),
            MoveDown => self.pc_move_by(0, 1),
            MoveRight => self.pc_move_by(-1, 0),
            MoveLeft => self.pc_move_by(1, 0),
        };
    }

    pub fn pc_move_by(&mut self, x: i32, y: i32) -> bool {
        let (x, y) = {
            let actor = self.pc();
            let x = x + (*actor).location.x as i32;
            let y = y + (*actor).location.y as i32;
            
            if x < 0 || y < 0 { return false; }
            let x = x as usize;
            let y = y as usize;

            let area_state = (*actor).location.area_state.borrow();

            if !area_state.is_passable(self.pc(), x, y) { return false; }

            (x, y)
        };

        return (*self.pc_mut()).move_to(x, y);
    }

    pub fn add_actor(&mut self, actor: &'a Actor, x: usize,
                     y: usize) -> bool {
        let location = Location::new(x, y, Rc::clone(&self.area_state));
        self.area_state_mut().add_actor(actor, location)
    }
}

struct Location<'a> {
    x: usize,
    y: usize,
    area_state: Rc<RefCell<AreaState<'a>>>
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

    fn coords_valid(&self, x: usize, y: usize) -> bool{
        if !self.area_state.borrow().area.coords_valid(x, y) { return false; }
        true
    }
}
