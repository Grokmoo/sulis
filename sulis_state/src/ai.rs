//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::rc::Rc;
use std::cell::RefCell;

use animation::WaitAnimation;
use sulis_core::config::CONFIG;
use {EntityState, GameState};

pub struct AI {
    ai: Option<EntityAI>,
}

impl AI {
    pub fn new() -> AI {
        AI {
            ai: None,
        }
    }

    pub fn update(&mut self, entity: Rc<RefCell<EntityState>>) {
        if entity.borrow().is_pc() {
            self.ai = None;
            return;
        }

        let assign = match self.ai {
            None => true,
            Some(ref ai) => !ai.is_for_entity(&entity),
        };

        if assign {
            debug!("Initialize round AI for '{}'", entity.borrow().actor.actor.name);
            self.ai = Some(EntityAI::new(&entity));
        }

        if let Some(ref mut ai) = self.ai {
            if !GameState::has_active_animations(&ai.entity) {
                ai.next();
                ai.take_action();
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Init,
    Wait(u32),
    Move,
    Attack,
    End,
    Complete,
}

const MAX_ACTIONS: u32 = 10;

struct EntityAI {
    entity: Rc<RefCell<EntityState>>,
    state: State,
    actions_taken_this_turn: u32,
}
impl EntityAI {
    fn new(entity: &Rc<RefCell<EntityState>>) -> EntityAI {
        EntityAI {
            entity: Rc::clone(entity),
            state: State::Init,
            actions_taken_this_turn: 0,
        }
    }

    fn next(&mut self) {
        use self::State::*;
        self.state = match self.state {
            Init => Wait(20),
            Wait(_) => self.pick_next_action(),
            Move | Attack => Wait(10),
            End => Complete,
            Complete => Complete,
        };

        info!("AI for '{}' transitioned into '{:?}'", self.entity.borrow().actor.actor.name, self.state);
    }

    fn pick_next_action(&self) -> State {
        if self.actions_taken_this_turn > MAX_ACTIONS {
            // guards against any infinite loop
            return State::End;
        }

        let entity = self.entity.borrow();
        let pc = GameState::pc();
        let area_state = GameState::area_state();
        let area = Rc::clone(&area_state.borrow().area);

        // TODO handle the case where ai is in range but cannot actually attack due to visibility
        // or other restrictions - find a path to the target, check each point along the path until
        // a good one is found
        if !entity.can_reach(&pc) && GameState::can_move_towards(&self.entity, &pc) {
            State::Move
        } else if entity.can_attack(&pc, &area) {
            State::Attack
        } else {
            State::End
        }
    }

    fn take_action(&mut self) {
        self.actions_taken_this_turn += 1;

        use self::State::*;
        match self.state {
            Init => (),
            Wait(time) => self.do_wait(time),
            Move => self.do_move(),
            Attack => self.do_attack(),
            End => self.do_end(),
            Complete => (),
        };
    }

    fn is_for_entity(&self, entity: &Rc<RefCell<EntityState>>) -> bool {
        *self.entity.borrow() == *entity.borrow()
    }

    fn do_wait(&self, time: u32) {
        debug!("AI for '{}' is waiting.", self.entity.borrow().actor.actor.name);
        let wait_time = CONFIG.display.animation_base_time_millis * time;
        let anim = WaitAnimation::new(&self.entity, wait_time);
        GameState::add_animation(Box::new(anim));
    }

    fn do_move(&self) {
        debug!("AI for '{}' is moving.", self.entity.borrow().actor.actor.name);
        let pc = GameState::pc();
        GameState::move_towards(&self.entity, &pc);
    }

    fn do_attack(&self) {
        debug!("AI for '{}' is attacking.", self.entity.borrow().actor.actor.name);
        let pc = GameState::pc();
        EntityState::attack(&self.entity, &pc);
    }

    fn do_end(&self) {
        debug!("AI for '{}' is ending.", self.entity.borrow().actor.actor.name);
        let area_state = GameState::area_state();
        area_state.borrow_mut().turn_timer.next();
    }
}
