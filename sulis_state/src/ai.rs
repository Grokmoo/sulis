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

use rlua;

use sulis_core::config::CONFIG;
use script::script_callback;
use {animation::Anim, EntityState, GameState};

pub struct AI {
    ai: Option<EntityAI>,
    next_state: State,
}

impl AI {
    pub fn new() -> AI {
        AI {
            ai: None,
            next_state: State::Run,
        }
    }

    pub fn update(&mut self, entity: Rc<RefCell<EntityState>>) {
        if GameState::is_modal_locked() { return; }

        if entity.borrow().is_party_member() {
            self.ai = None;
            return;
        }

        let assign = match self.ai {
            None => true,
            Some(ref ai) => !Rc::ptr_eq(&ai.entity, &entity),
        };

        if assign {
            debug!("Initialize round AI for '{}'", entity.borrow().actor.actor.name);
            self.ai = Some(EntityAI::new(&entity));
            self.next_state = State::Wait(20);
        }

        if let Some(ref mut ai) = self.ai {
            if GameState::has_blocking_animations(&ai.entity) { return; }

            self.next_state = match self.next_state {
                State::Run => {
                    ai.run_script()
                }, State::Wait(time) => {
                    ai.wait(time);
                    State::Run
                },
                State::End => {
                    debug!("AI for '{}' is ending.", ai.entity.borrow().actor.actor.name);
                    let turn_mgr = GameState::turn_manager();
                    let cbs = turn_mgr.borrow_mut().next();
                    script_callback::fire_round_elapsed(cbs);
                    State::Run
                }
            };
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum State {
    Run,
    Wait(u32),
    End,
}

impl rlua::UserData for State {}

const MAX_ACTIONS: u32 = 10;

struct EntityAI {
    entity: Rc<RefCell<EntityState>>,
    actions_taken_this_turn: u32,
}

impl EntityAI {
    fn new(entity: &Rc<RefCell<EntityState>>) -> EntityAI {
        EntityAI {
            entity: Rc::clone(entity),
            actions_taken_this_turn: 0,
        }
    }

    fn wait(&self, time: u32) {
        debug!("AI for '{}' is waiting.", self.entity.borrow().actor.actor.name);
        let wait_time = CONFIG.display.animation_base_time_millis * time;
        let anim = Anim::new_wait(&self.entity, wait_time);
        GameState::add_animation(anim);
    }

    fn run_script(&mut self) -> State {
        if self.actions_taken_this_turn == MAX_ACTIONS { return State::End }

        if self.entity.borrow().actor.actor.ai.is_none() {
            return State::End;
        }

        self.actions_taken_this_turn += 1;

        info!("ai action script");
        GameState::execute_ai_script(&self.entity, "ai_action")
    }

    // fn pick_next_action(&mut self) -> State {
    //     if self.actions_taken_this_turn > MAX_ACTIONS {
    //         // guards against any infinite loop
    //         return State::End;
    //     }
    //
    //     self.pick_target();
    //     let entity = self.entity.borrow();
    //     let area_state = GameState::area_state();
    //     let area_state = &area_state.borrow();
    //
    //     let target = match self.target {
    //         None => return State::End,
    //         Some(ref target) => target,
    //     };
    //
    //     // TODO handle the case where ai is in range but cannot actually attack due to visibility
    //     // or other restrictions - find a path to the target, check each point along the path until
    //     // a good one is found
    //     if !entity.can_reach(target) && GameState::can_move_towards(&self.entity, target) {
    //         State::Move
    //     } else if entity.can_attack(target, &area_state) {
    //         State::Attack
    //     } else {
    //         State::End
    //     }
    // }
    //
    // fn pick_target(&mut self) {
    //     let mgr = GameState::turn_manager();
    //     let mgr = mgr.borrow();
    //
    //     let area_id = self.entity.borrow().location.area_id.to_string();
    //
    //     let mut dists = Vec::new();
    //     for target in mgr.entity_iter() {
    //         if !self.entity.borrow().is_hostile(&target) { continue; }
    //         if target.borrow().actor.stats.hidden { continue; }
    //         if !target.borrow().location.is_in_area_id(&area_id) { continue; }
    //
    //         let dist = self.entity.borrow().dist_to_entity(&target);
    //         dists.push((target, (dist * 100.0) as i32));
    //     }
    //
    //     if dists.is_empty() {
    //         self.target = None;
    //         return;
    //     }
    //
    //     dists.sort_by_key(|k| k.1);
    //
    //     self.target = Some(dists.remove(0).0);
    // }
    //
    // fn take_action(&mut self) {
    //     self.actions_taken_this_turn += 1;
    //
    //     use self::State::*;
    //     match self.state {
    //         Init => (),
    //         Wait(time) => self.do_wait(time),
    //         Move => self.do_move(),
    //         Attack => self.do_attack(),
    //         End => self.do_end(),
    //         Complete => (),
    //     };
    // }
    //
    // fn is_for_entity(&self, entity: &Rc<RefCell<EntityState>>) -> bool {
    //     *self.entity.borrow() == *entity.borrow()
    // }
    //
    // fn do_wait(&self, time: u32) {
    //     debug!("AI for '{}' is waiting.", self.entity.borrow().actor.actor.name);
    //     let wait_time = CONFIG.display.animation_base_time_millis * time;
    //     let anim = Anim::new_wait(&self.entity, wait_time);
    //     GameState::add_animation(anim);
    // }
    //
    // fn do_move(&self) {
    //     let target = match self.target {
    //         None => return,
    //         Some(ref target) => target,
    //     };
    //
    //     debug!("AI for '{}' is moving.", self.entity.borrow().actor.actor.name);
    //     GameState::move_towards(&self.entity, target);
    // }
    //
    // fn do_attack(&self) {
    //     let target = match self.target {
    //         None => return,
    //         Some(ref target) => target,
    //     };
    //
    //     debug!("AI for '{}' is attacking.", self.entity.borrow().actor.actor.name);
    //     EntityState::attack(&self.entity, target, None, true);
    // }
    //
    // fn do_end(&self) {
    //     debug!("AI for '{}' is ending.", self.entity.borrow().actor.actor.name);
    //     let turn_mgr = GameState::turn_manager();
    //     let cbs = turn_mgr.borrow_mut().next();
    //     script_callback::fire_round_elapsed(cbs);
    // }
}
