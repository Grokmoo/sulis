use std::rc::Rc;
use std::cell::RefCell;

use animation::WaitAnimation;
use grt::config::CONFIG;
use state::{EntityState, GameState};

pub struct AI {
    entity_ai: Option<EntityAI>,
}

impl AI {
    pub fn new() -> AI {
        AI {
            entity_ai: None,
        }
    }

    pub fn update(&mut self, entity: Rc<RefCell<EntityState>>) {
        if entity.borrow().is_pc() {
            self.entity_ai = None;
            return;
        }

        let assign = match self.entity_ai {
            None => true,
            Some(ref ai) => {
                if !ai.is_for_entity(&entity) {
                    true
                } else {
                    false
                }
            },
        };

        if assign {
            debug!("Initialize round AI for '{}'", entity.borrow().actor.actor.name);
            self.entity_ai = Some(EntityAI::new(entity));
        }

        if let Some(ref mut ai) = self.entity_ai {
            ai.update();
        }
    }
}

struct EntityAI {
    entity: Rc<RefCell<EntityState>>,
    initial_wait_done: bool,
    move_done: bool,
    attack_done: bool,
}

impl EntityAI {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> EntityAI {
        EntityAI {
            entity,
            initial_wait_done: false,
            move_done: false,
            attack_done: false,
        }
    }

    pub fn is_for_entity(&self, entity: &Rc<RefCell<EntityState>>) -> bool {
        *self.entity.borrow() == *entity.borrow()
    }

    pub fn update(&mut self) {
        if GameState::has_active_animations(&self.entity) {
            return;
        }

        if !self.initial_wait_done {
            let wait_time = CONFIG.display.animation_base_time_millis * 40;
            let anim = WaitAnimation::new(&self.entity, wait_time);
            GameState::add_animation(Box::new(anim));
            self.initial_wait_done = true;
            return;
        }

        if !self.move_done {
            let pc = GameState::pc();
            GameState::entity_move_towards(&self.entity, &pc);
            self.move_done = true;
            return;
        }

        if !self.attack_done {
            let pc = GameState::pc();
            if !self.entity.borrow().can_attack(&pc) {
                self.attack_done = true;
                return;
            }

            self.entity.borrow_mut().attack(&pc);
            let anim = WaitAnimation::new(&self.entity, CONFIG.display.animation_base_time_millis * 20);
            GameState::add_animation(Box::new(anim));
            return;
        }

        let area_state = GameState::area_state();
        area_state.borrow_mut().turn_timer.next();
    }
}
