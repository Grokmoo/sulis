mod area_state;
pub use self::area_state::AreaState;

mod entity_state;
pub use self::entity_state::EntityState;

mod actor_state;
pub use self::actor_state::ActorState;

mod item_state;
pub use self::item_state::ItemState;

pub mod inventory;
pub use self::inventory::Inventory;

mod location;
pub use self::location::Location;

mod path_finder;
use self::path_finder::PathFinder;

use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;

use grt::resource::{Actor, ResourceSet};
use grt::config::CONFIG;
use grt::util::Point;
use animation::{Animation, MoveAnimation};

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

pub struct GameState {
    pub area_state: Rc<RefCell<AreaState>>,
    pub pc: Rc<RefCell<EntityState>>,
    pub should_exit: bool,

    animations: Vec<Box<Animation>>,
    path_finder: PathFinder,
}

impl GameState {
    pub fn init() -> Result<(), Error> {
        let game_state = GameState::new()?;

        STATE.with(|state| {
            *state.borrow_mut() = Some(game_state);
        });

        Ok(())
    }

    pub fn transition(area_id: &Option<String>, x: i32, y: i32) {
        let p = Point::new(x, y);
        info!("Area transition to {:?} at {},{}", area_id, x, y);

        if let &Some(ref area_id) = area_id {
            let area_state = match GameState::setup_area_state(area_id) {
                Ok(state) => state,
                Err(_) => {
                    error!("Unable to transition to '{}'", &area_id);
                    return;
                }
            };

            if !GameState::check_location(&p, &area_state) {
                return;
            }

            STATE.with(|state| {
                let path_finder = PathFinder::new(Rc::clone(&area_state));
                state.borrow_mut().as_mut().unwrap().path_finder = path_finder;
                state.borrow_mut().as_mut().unwrap().area_state = area_state;
            });
        } else {
            if !GameState::check_location(&p, &GameState::area_state()) {
                return;
            }
        }

        {
            let pc = GameState::pc();
            let old_area_state = &pc.borrow().location.area_state;
            old_area_state.borrow_mut().remove_entity(&pc);
        }

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            let location = Location::new(x, y, Rc::clone(&state.area_state));
            state.area_state.borrow_mut().add_entity(Rc::clone(&state.pc), location);
        });
    }

    fn check_location(p: &Point, area_state: &Rc<RefCell<AreaState>>) -> bool {
        let location = Location::from_point(p, Rc::clone(area_state));
        if !location.coords_valid(location.x, location.y) {
            error!("Location coordinates {},{} are not valid for area {}",
                   location.x, location.y, location.area_id);
            return false;
        }

        true
    }

    fn setup_area_state(area_id: &str) -> Result<Rc<RefCell<AreaState>>, Error> {
        debug!("Setting up area state from {}", &area_id);
        let area = ResourceSet::get_area(&area_id);
        let area = match area {
            Some(a) => a,
            None => {
                error!("Area '{}' not found", &area_id);
                return Err(Error::new(ErrorKind::NotFound, "Unable to create area."));
            }
        };
        let area_state = Rc::new(RefCell::new(AreaState::new(area)));
        AreaState::populate(&area_state);

        Ok(area_state)
    }

    fn new() -> Result<GameState, Error> {
        let game = ResourceSet::get_game();

        let area_state = GameState::setup_area_state(&game.starting_area)?;

        debug!("Setting up PC {}, with {:?}", &game.pc, &game.starting_location);
        let location = Location::from_point(&game.starting_location,
                                            Rc::clone(&area_state));

        if !location.coords_valid(location.x, location.y) {
            error!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }

        let pc = ResourceSet::get_actor(&game.pc);
        let pc = match pc {
            Some(a) => a,
            None => {
                error!("Player character '{}' not found", &game.pc);
                return Err(Error::new(ErrorKind::NotFound,
                                      "Unable to create player character."));
            }
        };

        if !area_state.borrow_mut().add_actor(pc, location) {
            error!("Player character starting location must be within \
                   area bounds and passable.");
            return Err(Error::new(ErrorKind::InvalidData,
                "Unable to add player character to starting area at starting location"));
        }

        let pc_state = Rc::clone(area_state.borrow().get_last_entity().unwrap());

        let path_finder = PathFinder::new(Rc::clone(&area_state));

        Ok(GameState {
            area_state: area_state,
            path_finder: path_finder,
            pc: pc_state,
            animations: Vec::new(),
            should_exit: false,
        })
    }

    pub fn is_exit() -> bool {
        STATE.with(|s| s.borrow().as_ref().unwrap().should_exit)
    }

    pub fn set_exit() -> bool {
        trace!("Setting state exit flag.");
        STATE.with(|s| s.borrow_mut().as_mut().unwrap().should_exit = true);
        true
    }

    pub fn area_state() -> Rc<RefCell<AreaState>> {
        STATE.with(|s| Rc::clone(&s.borrow().as_ref().unwrap().area_state))
    }

    pub fn update() {
        STATE.with(|s| {
            s.borrow_mut().as_mut().unwrap().animations.retain(|anim| anim.update());
        });
    }

    pub fn pc() -> Rc<RefCell<EntityState>> {
        STATE.with(|s| Rc::clone(&s.borrow().as_ref().unwrap().pc))
    }

    pub fn can_pc_move_to(x: i32, y: i32) -> bool {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            let path = state.path_finder.find(state.pc.borrow(), x, y);

            path.is_some()
        })
    }

    pub fn pc_move_to(x: i32, y: i32) -> bool {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            trace!("Moving pc to {},{}", x, y);
            let path = state.path_finder.find(state.pc.borrow(), x, y);

            if let None = path {
                return false;
            }
            let path = path.unwrap();
            let entity = Rc::clone(&state.pc);
            for anim in state.animations.iter_mut() {
                anim.check(&entity);
            }
            let anim = MoveAnimation::new(entity, path,
                                          CONFIG.display.animation_base_time_millis);
            state.animations.push(Box::new(anim));
            true
        })
    }

    pub fn add_actor(actor: Rc<Actor>, x: i32, y: i32) -> bool {
        STATE.with(|s| {
            let area_state = Rc::clone(&s.borrow_mut().as_mut().unwrap().area_state);
            let location = Location::new(x, y, Rc::clone(&area_state));

            let result = area_state.borrow_mut().add_actor(actor, location);
            result
        })
    }
}
