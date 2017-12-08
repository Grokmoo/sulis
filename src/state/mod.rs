mod area_state;
pub use self::area_state::AreaState;

mod entity_state;
pub use self::entity_state::EntityState;

mod actor_state;
pub use self::actor_state::ActorState;

mod location;
pub use self::location::Location;

mod cursor;
pub use self::cursor::Cursor;

mod path_finder;

use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use resource::ResourceSet;
use resource::Actor;
use config::Config;
use io::mouse_event;
use io::{MouseEvent, KeyboardEvent, InputAction, TextRenderer};
use animation::{Animation, MoveAnimation};
use ui::WidgetBase;

pub struct GameState<'a> {
    config: Config,
    pub area_state: Rc<RefCell<AreaState<'a>>>,
    pub pc: Rc<RefCell<EntityState<'a>>>,
    pub cursor: Cursor,
    animations: Vec<Box<Animation + 'a>>,

    pub should_exit: bool,
}

impl<'a> GameState<'a> {
    pub fn new(config: Config, resources: &'a ResourceSet) -> Result<GameState<'a>, Error> {
        let game = &resources.game;

        let area = resources.get_area(&game.starting_area);
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
        let x = *game.starting_location.get(0).unwrap();
        let y = *game.starting_location.get(1).unwrap();
        let location = Location::new(x, y, Rc::clone(&area_state));

        if !location.coords_valid(location.x, location.y) {
            eprintln!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }

        let pc = resources.get_actor(&game.pc);
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

        let pc_state = Rc::clone(area_state.borrow().entities.last().unwrap());

        let display_width = config.display.width;
        let display_height = config.display.height;
        let cursor_char = config.display.cursor_char;

        Ok(GameState {
            config: config,
            area_state: area_state,
            pc: pc_state,
            cursor: Cursor {
                x: x as i32,
                y: y as i32,
                max_x: display_width,
                max_y: display_height,
                c: cursor_char,
            },
            animations: Vec::new(),
            should_exit: false,
        })
    }

    pub fn set_exit(&mut self) -> bool{
        self.should_exit = true;
        true
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer,
                          root: Ref<WidgetBase>, millis: u32) {
        root.draw_text_mode(renderer);

        self.cursor.draw_text_mode(renderer, millis);
    }

    pub fn cursor_move_by(&mut self, root: RefMut<WidgetBase>, x: i32, y: i32) -> bool {
        if self.cursor.move_by(x, y) {
            let event = MouseEvent::new(mouse_event::Kind::Move(x, y),
                 self.cursor.x, self.cursor.y);
            return root.dispatch_event(self, event);
        }

        false
    }

    pub fn cursor_click(&mut self, root: RefMut<WidgetBase>) -> bool {
        let x = self.cursor.x;
        let y = self.cursor.y;

        let event = MouseEvent::new(mouse_event::Kind::LeftClick, x, y);
        root.dispatch_event(self, event)
    }

    pub fn update(&mut self) {
        self.animations.retain(|anim| anim.update());
    }

    pub fn handle_keyboard_input(&mut self, input: KeyboardEvent,
                                 root: RefMut<WidgetBase>) {
        let action = {
            let action = self.config.get_input_action(input);

            if let None = action { return; }

            *action.unwrap()
        };

        InputAction::fire_action(action, self, root);
    }

    pub fn pc(&self) -> Ref<EntityState<'a>> {
        self.pc.borrow()
    }

    pub fn pc_mut(&mut self) -> RefMut<EntityState<'a>> {
        self.pc.borrow_mut()
    }

    pub fn pc_move_to(&mut self, x: usize, y: usize) -> bool {
        let path = self.area_state.borrow_mut().find_path(self.pc(), x, y);

        if let None = path {
            return false;
        }
        let path = path.unwrap();
        let anim = MoveAnimation::new(Rc::clone(&self.pc),
            path, self.config.display.animation_base_time_millis);
        self.animations.push(Box::new(anim));
        true
    }

    pub fn pc_move_by(&mut self, x: i32, y: i32) -> bool {
        let (x, y) = {
            let entity = self.pc();
            let x = x + (*entity).location.x as i32;
            let y = y + (*entity).location.y as i32;

            if x < 0 || y < 0 { return false; }
            let x = x as usize;
            let y = y as usize;

            let area_state = (*entity).location.area_state.borrow();

            if !area_state.is_passable(self.pc(), x, y) { return false; }

            (x, y)
        };

        return (*self.pc_mut()).move_to(x, y);
    }

    pub fn add_actor(&mut self, actor: Rc<Actor>, x: usize,
                     y: usize) -> bool {
        let location = Location::new(x, y, Rc::clone(&self.area_state));

        self.area_state.borrow_mut().add_actor(actor, location)
    }
}
