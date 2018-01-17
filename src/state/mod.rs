mod ai;
pub use self::ai::AI;

mod area_state;
pub use self::area_state::AreaState;

mod change_listener;
pub use self::change_listener::ChangeListener;
pub use self::change_listener::ChangeListenerList;

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

mod turn_timer;
pub use self::turn_timer::TurnTimer;

use std::time;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;

use grt::config::CONFIG;
use grt::util::Point;
use grt::io::MainLoopUpdater;
use animation::{self, Animation, MoveAnimation};
use module::Module;

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
    static AI: RefCell<AI> = RefCell::new(AI::new());
}

pub struct GameStateMainLoopUpdater { }

impl MainLoopUpdater for GameStateMainLoopUpdater {
    fn update(&self) {
        GameState::update();
    }

    fn is_exit(&self) -> bool {
        GameState::is_exit()
    }
}

pub struct GameState {
    areas: HashMap<String, Rc<RefCell<AreaState>>>,
    area_state: Rc<RefCell<AreaState>>,
    pc: Rc<RefCell<EntityState>>,
    should_exit: bool,
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
            // check if area state has already been loaded
            let area_state = GameState::get_area_state(area_id);
            let area_state = match area_state {
                Some(area_state) => area_state,
                None => match GameState::setup_area_state(area_id) {
                    // area state has not already been loaded, try to load it
                    Ok(area_state) => {
                        STATE.with(|state| {
                            let mut state = state.borrow_mut();
                            let state = state.as_mut().unwrap();
                            state.areas.insert(area_id.to_string(), Rc::clone(&area_state));
                        });

                        area_state
                    }, Err(e) => {
                        error!("Unable to transition to '{}'", &area_id);
                        error!("{}", e);
                        return;
                    }
                }
            };

            if !GameState::check_location(&p, &area_state) {
                return;
            }

            STATE.with(|state| {
                let path_finder = PathFinder::new(&area_state.borrow().area);
                state.borrow_mut().as_mut().unwrap().path_finder = path_finder;
                state.borrow_mut().as_mut().unwrap().area_state = area_state;
            });
        } else {
            if !GameState::check_location(&p, &GameState::area_state()) {
                return;
            }
        }

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            {
                let area_id = state.pc.borrow().location.area_id.to_string();
                state.areas.get(&area_id).unwrap().borrow_mut().remove_entity(&state.pc);
            }

            let location = Location::new(x, y, &state.area_state.borrow().area);
            state.area_state.borrow_mut().add_entity(Rc::clone(&state.pc), location);
        });
    }

    fn check_location(p: &Point, area_state: &Rc<RefCell<AreaState>>) -> bool {
        let location = Location::from_point(p, &area_state.borrow().area);
        if !location.coords_valid(location.x, location.y) {
            error!("Location coordinates {},{} are not valid for area {}",
                   location.x, location.y, location.area_id);
            return false;
        }

        true
    }

    fn setup_area_state(area_id: &str) -> Result<Rc<RefCell<AreaState>>, Error> {
        debug!("Setting up area state from {}", &area_id);

        let area = Module::area(&area_id);
        let area = match area {
            Some(a) => a,
            None => {
                error!("Area '{}' not found", &area_id);
                return Err(Error::new(ErrorKind::NotFound, "Unable to create area."));
            }
        };
        let area_state = Rc::new(RefCell::new(AreaState::new(area)));
        area_state.borrow_mut().populate();

        Ok(area_state)
    }

    fn new() -> Result<GameState, Error> {
        let game = Module::game();

        let area_state = GameState::setup_area_state(&game.starting_area)?;

        debug!("Setting up PC {}, with {:?}", &game.pc, &game.starting_location);
        let location = Location::from_point(&game.starting_location, &area_state.borrow().area);

        if !location.coords_valid(location.x, location.y) {
            error!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }

        let pc = Module::actor(&game.pc);
        let pc = match pc {
            Some(a) => a,
            None => {
                error!("Player character '{}' not found", &game.pc);
                return Err(Error::new(ErrorKind::NotFound,
                                      "Unable to create player character."));
            }
        };

        if !area_state.borrow_mut().add_actor(pc, location, true) {
            error!("Player character starting location must be within \
                   area bounds and passable.");
            return Err(Error::new(ErrorKind::InvalidData,
                "Unable to add player character to starting area at starting location"));
        }

        let pc_state = Rc::clone(area_state.borrow().get_last_entity().unwrap());

        let path_finder = PathFinder::new(&area_state.borrow().area);

        let mut areas: HashMap<String, Rc<RefCell<AreaState>>> = HashMap::new();
        areas.insert(game.starting_area.to_string(), Rc::clone(&area_state));

        Ok(GameState {
            areas,
            area_state: area_state,
            path_finder: path_finder,
            pc: pc_state,
            animations: Vec::new(),
            should_exit: false,
        })
    }

    fn get_area_state(id: &str) -> Option<Rc<RefCell<AreaState>>> {
        STATE.with(|s| {
            match s.borrow().as_ref().unwrap().areas.get(id) {
                None => None,
                Some(area_state) => Some(Rc::clone(&area_state)),
            }
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
        let active_entity = STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            let mut area_state = state.area_state.borrow_mut();

            let mut i = 0;
            while i < state.animations.len() {
                let retain = state.animations[i].update(&mut area_state);

                if retain {
                    i += 1;
                } else {
                    state.animations.remove(i);
                }
            }

            let result = match area_state.update() {
                None => Rc::clone(&state.pc),
                Some(entity) => Rc::clone(entity),
            };

            if state.pc.borrow().actor.is_dead() {
                area_state.turn_timer.set_active(false);
            }

            result
        });

        AI.with(|ai| {
            let mut ai = ai.borrow_mut();
            ai.update(active_entity);
        });
    }

    pub fn has_active_animations(entity: &Rc<RefCell<EntityState>>) -> bool {
        STATE.with(|s| {
            let state = s.borrow();
            let state = state.as_ref().unwrap();

            for anim in state.animations.iter() {
                if *anim.get_owner().borrow() == *entity.borrow() {
                    return true;
                }
            }
            false
        })
    }

    pub fn add_animation(anim: Box<Animation>) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            state.animations.push(anim);
        });
    }

    /// Returns true if the PC has the current turn, false otherwise
    pub fn is_pc_current() -> bool {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        if let Some(entity) = area_state.turn_timer.current() {
            return entity.borrow().is_pc();
        }
        false
    }

    pub fn pc() -> Rc<RefCell<EntityState>> {
        STATE.with(|s| Rc::clone(&s.borrow().as_ref().unwrap().pc))
    }

    pub fn entity_move_towards(entity: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) -> bool {
        let (target_x, target_y) = {
            let target = target.borrow();
            (target.location.x + target.size.size / 2, target.location.y + target.size.size / 2)
        };

        let dist = entity.borrow().size.size / 2 + target.borrow().size.size / 2 + 1;
        GameState::entity_move_to_internal(entity, target_x as f32, target_y as f32, dist as f32)
    }

    pub fn entity_move_to(entity: &Rc<RefCell<EntityState>>, x: i32, y: i32) -> bool {
        GameState::entity_move_to_internal(entity, x as f32, y as f32, 0.0)
    }

    fn entity_move_to_internal(entity: &Rc<RefCell<EntityState>>, x: f32, y: f32, dist: f32) -> bool {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            let area_state = state.area_state.borrow();
            trace!("Moving '{}' to {},{}", entity.borrow().actor.actor.name, x, y);

            let start_time = time::Instant::now();
            let path = match state.path_finder.find(&area_state, entity.borrow(), x, y, dist) {
                None => return false,
                Some(path) => path,
            };
            debug!("Path finding complete in {} secs", animation::format_elapsed_secs(start_time.elapsed()));

            for anim in state.animations.iter_mut() {
                anim.check(entity);
            }
            let entity = Rc::clone(entity);
            let anim = MoveAnimation::new(entity, path, CONFIG.display.animation_base_time_millis);
            state.animations.push(Box::new(anim));
            true
        })
    }
}
