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

extern crate rlua;
extern crate rand;

extern crate sulis_core;
extern crate sulis_module;
extern crate sulis_rules;
#[macro_use] extern crate log;

mod ai;
pub use self::ai::AI;

mod ability_state;
pub use self::ability_state::AbilityState;

mod actor_state;
pub use self::actor_state::ActorState;

pub mod animation;
use self::animation::{Animation, MoveAnimation};

mod area_feedback_text;
use self::area_feedback_text::AreaFeedbackText;

mod area_state;
pub use self::area_state::AreaState;

mod change_listener;
pub use self::change_listener::ChangeListener;
pub use self::change_listener::ChangeListenerList;

mod effect;
pub use self::effect::Effect;

mod entity_state;
pub use self::entity_state::EntityState;
pub use self::entity_state::AreaDrawable;

mod entity_texture_cache;
pub use self::entity_texture_cache::EntityTextureCache;
pub use self::entity_texture_cache::EntityTextureSlot;

mod item_state;
pub use self::item_state::ItemState;

pub mod inventory;
pub use self::inventory::Inventory;

pub mod item_list;
pub use self::item_list::ItemList;

mod location;
pub use self::location::Location;

mod los_calculator;
pub use self::los_calculator::calculate_los;
pub use self::los_calculator::has_visibility;

mod merchant;
pub use self::merchant::Merchant;

mod path_finder;
use self::path_finder::PathFinder;

mod prop_state;
pub use self::prop_state::PropState;

mod script;
pub use self::script::ScriptState;
pub use self::script::targeter::Targeter;
pub use self::script::ScriptCallback;

mod turn_timer;
pub use self::turn_timer::TurnTimer;
pub use self::turn_timer::ROUND_TIME_MILLIS;

use std::time;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;

use sulis_rules::HitKind;
use sulis_core::config::CONFIG;
use sulis_core::util::{self, Point};
use sulis_core::io::{GraphicsRenderer, MainLoopUpdater};
use sulis_core::ui::{Widget};
use sulis_module::{Ability, Actor, Module, OnTrigger, area::{Trigger, TriggerKind}};

use script::ScriptEntitySet;
use script::script_callback::ScriptHitKind;

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
    static AI: RefCell<AI> = RefCell::new(AI::new());
    static ENTERING_COMBAT: RefCell<bool> = RefCell::new(false);
    static SCRIPT: ScriptState = ScriptState::new();
    static ANIMATIONS: RefCell<Vec<Box<Animation>>> = RefCell::new(Vec::new());
    static ANIMS_TO_ADD: RefCell<Vec<Box<Animation>>> = RefCell::new(Vec::new());
}

pub struct GameStateMainLoopUpdater { }

impl MainLoopUpdater for GameStateMainLoopUpdater {
    fn update(&self, root: &Rc<RefCell<Widget>>, millis: u32) {
        GameState::update(root, millis);
    }

    fn is_exit(&self) -> bool {
        GameState::is_exit()
    }
}

pub struct UICallback {
    pub on_trigger: OnTrigger,
    pub parent: Rc<RefCell<EntityState>>,
    pub target: Rc<RefCell<EntityState>>,
}

pub struct GameState {
    areas: HashMap<String, Rc<RefCell<AreaState>>>,
    area_state: Rc<RefCell<AreaState>>,
    selected: Vec<Rc<RefCell<EntityState>>>,
    party: Vec<Rc<RefCell<EntityState>>>,

    // listener returns the first selected party member
    party_listeners: ChangeListenerList<Option<Rc<RefCell<EntityState>>>>,
    should_exit: bool,
    path_finder: PathFinder,
    ui_callbacks: Vec<UICallback>,
}

macro_rules! exec_script {
    ($func:ident: $($x:ident),*) => {
        let start_time = time::Instant::now();

        let result: Result<(), rlua::Error> = SCRIPT.with(|script_state| {
            script_state.$func($($x, )*)
        });

        if let Err(e) = result {
            warn!("Error executing lua script function");
            warn!("{}", e);
        }

        info!("Script execution time: {}", util::format_elapsed_secs(start_time.elapsed()));
    }
}

impl GameState {
    pub fn set_selected_party_member(entity: Rc<RefCell<EntityState>>) {
        GameState::select_party_members(vec![entity]);
    }

    pub fn clear_selected_party_member() {
        GameState::select_party_members(Vec::new());
    }

    pub fn select_party_members(members: Vec<Rc<RefCell<EntityState>>>) {
        for member in members.iter() {
            if !member.borrow().is_party_member() {
                warn!("Attempted to select non-party member {}", member.borrow().actor.actor.id);
            }
        }

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.selected.clear();
            // add in party member order
            for party_member in state.party.iter() {
                for member in members.iter() {
                    if Rc::ptr_eq(party_member, member) {
                        state.selected.push(Rc::clone(member));
                    }
                }
            }

            let entity = match state.selected.first() {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            };
            state.party_listeners.notify(&entity);
        })
    }

    pub fn selected() -> Vec<Rc<RefCell<EntityState>>> {
        STATE.with(|s| s.borrow().as_ref().unwrap().selected.clone())
    }

    pub fn add_party_member(entity: Rc<RefCell<EntityState>>) {
        info!("Add party member {}", entity.borrow().actor.actor.id);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            if !state.area_state.borrow().turn_timer.is_active() {
                entity.borrow_mut().actor.init_turn();
            }

            entity.borrow_mut().set_party_member(true);
            state.area_state.borrow_mut().compute_pc_visibility(&entity, 0, 0);
            state.party.push(Rc::clone(&entity));

            let entity = match state.selected.first() {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            };
            state.party_listeners.notify(&entity);
        })
    }

    pub fn add_party_listener(listener: ChangeListener<Option<Rc<RefCell<EntityState>>>>) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.party_listeners.add(listener);
        })
    }

    pub fn player() -> Rc<RefCell<EntityState>> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            Rc::clone(&state.party[0])
        })
    }

    pub fn party() -> Vec<Rc<RefCell<EntityState>>> {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.party.clone()
        })
    }

    pub fn execute_ability_on_activate(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>) {
        exec_script!(ability_on_activate: parent, ability);
    }

    pub fn execute_ability_on_target_select(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                            targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                                            selected_point: Point) {
        exec_script!(ability_on_target_select: parent, ability, targets, selected_point);
    }

    pub fn execute_ability_after_attack(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                        targets: ScriptEntitySet,
                                        kind: HitKind, func: &str) {
        let hit_kind = ScriptHitKind { kind };
        let t = Some(("hit", hit_kind));
        exec_script!(ability_script: parent, ability, targets, t, func);
    }

    pub fn execute_ability_script(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                  targets: ScriptEntitySet, func: &str) {
        let t: Option<(&str, usize)> = None;
        exec_script!(ability_script: parent, ability, targets, t, func);
    }

    pub fn execute_trigger_script(script_id: &str, func: &str, parent: &Rc<RefCell<EntityState>>,
                                  target: &Rc<RefCell<EntityState>>) {
        exec_script!(trigger_script: script_id, func, parent, target);
    }

    pub fn init(pc_actor: Rc<Actor>) -> Result<(), Error> {
        let game_state = GameState::new(pc_actor)?;

        STATE.with(|state| {
            *state.borrow_mut() = Some(game_state);
        });

        let pc = GameState::player();
        let area_state = GameState::area_state();
        area_state.borrow_mut().update_view_visibility();
        area_state.borrow_mut().push_scroll_to_callback(Rc::clone(&pc));
        area_state.borrow_mut().on_load_fired = true;
        let area_state = area_state.borrow();
        GameState::add_ui_callbacks_of_kind(&area_state.area.triggers, TriggerKind::OnCampaignStart, &pc, &pc);
        GameState::add_ui_callbacks_of_kind(&area_state.area.triggers, TriggerKind::OnAreaLoad, &pc, &pc);

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
                for entity in state.party.iter() {
                    let area_id = entity.borrow().location.area_id.to_string();
                    state.areas.get(&area_id).unwrap().borrow_mut().remove_entity(&entity);
                }
            }

            let location = Location::new(x, y, &state.area_state.borrow().area);

            for entity in state.party.iter() {
                // TODO find locations for all of the party members
                state.area_state.borrow_mut().add_entity(Rc::clone(entity), location.clone());
            }

            state.area_state.borrow_mut().push_scroll_to_callback(Rc::clone(&state.party[0]));

            let area_state = state.area_state.borrow();
            for entity in area_state.entity_iter() {
                entity.borrow_mut().clear_texture_cache();
            }
        });

        let area_state = GameState::area_state();
        let pc = GameState::player();
        let mut area_state = area_state.borrow_mut();
        if !area_state.on_load_fired {
            area_state.on_load_fired = true;
            GameState::add_ui_callbacks_of_kind(&area_state.area.triggers, TriggerKind::OnAreaLoad, &pc, &pc);
        }
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

    fn new(pc: Rc<Actor>) -> Result<GameState, Error> {
        let game = Module::game();

        let area_state = GameState::setup_area_state(&game.starting_area)?;

        debug!("Setting up PC {}, with {:?}", &pc.name, &game.starting_location);
        let location = Location::from_point(&game.starting_location, &area_state.borrow().area);

        if !location.coords_valid(location.x, location.y) {
            error!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Unable to create starting location."));
        }

        if !area_state.borrow_mut().add_actor(pc, location, true, None) {
            error!("Player character starting location must be within \
                   area bounds and passable.");
            return Err(Error::new(ErrorKind::InvalidData,
                "Unable to add player character to starting area at starting location"));
        }

        let pc_state = Rc::clone(area_state.borrow().get_last_entity().unwrap());
        pc_state.borrow_mut().actor.init_turn();

        let path_finder = PathFinder::new(&area_state.borrow().area);

        let mut areas: HashMap<String, Rc<RefCell<AreaState>>> = HashMap::new();
        areas.insert(game.starting_area.to_string(), Rc::clone(&area_state));

        let mut party = Vec::new();
        party.push(Rc::clone(&pc_state));

        let mut selected = Vec::new();
        selected.push(Rc::clone(&pc_state));

        Ok(GameState {
            areas,
            area_state: area_state,
            path_finder: path_finder,
            selected,
            party,
            party_listeners: ChangeListenerList::default(),
            should_exit: false,
            ui_callbacks: Vec::new(),
        })
    }

    pub fn add_ui_callback(cb: OnTrigger, parent: &Rc<RefCell<EntityState>>,
                           target: &Rc<RefCell<EntityState>>) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            let ui_cb = UICallback {
                on_trigger: cb,
                parent: Rc::clone(parent),
                target: Rc::clone(target),
            };
            state.ui_callbacks.push(ui_cb);
        })
    }

    pub fn add_ui_callbacks_of_kind(callbacks: &Vec<Trigger>, kind: TriggerKind,
                                    parent: &Rc<RefCell<EntityState>>, target: &Rc<RefCell<EntityState>>) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            for cb in callbacks.iter() {
                if cb.kind == kind {
                    let ui_cb = UICallback {
                        on_trigger: cb.on_activate.clone(),
                        parent: Rc::clone(parent),
                        target: Rc::clone(target),
                    };
                    state.ui_callbacks.push(ui_cb);
                }
            }
        })
    }

    pub fn check_get_ui_callback() -> Option<UICallback> {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            state.ui_callbacks.pop()
        })
    }

    fn check_clear_entering_combat() -> bool {
        ENTERING_COMBAT.with(|c| {
            let retval = {
                *c.borrow()
            };

            *c.borrow_mut() = false;
            retval
        })
    }

    pub fn set_entering_combat() {
        ENTERING_COMBAT.with(|c| *c.borrow_mut() = true);
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

    pub fn update(root: &Rc<RefCell<Widget>>, millis: u32) {
        let mut anims_to_add: Vec<Box<Animation>> = ANIMS_TO_ADD.with(|a| {
            let mut anims = a.borrow_mut();

            let to_add = anims.drain(0..).collect();

            to_add
        });

        ANIMATIONS.with(|a| {
            let mut anims = a.borrow_mut();

            anims.append(&mut anims_to_add);

            let mut i = 0;
            while i < anims.len() {
                let retain = anims[i].update(root);

                if retain {
                    i += 1;
                } else {
                    anims.remove(i);
                }
            }
        });

        let active_entity = STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            let mut area_state = state.area_state.borrow_mut();

            let result = area_state.update(millis);
            // TODO check for whole party death
            // if state.selected.borrow().actor.is_dead() {
            //     area_state.turn_timer.set_active(false);
            // }

            match result {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            }
        });

        // clear animations for the active entity when entering combat
        if GameState::check_clear_entering_combat() {
            ANIMATIONS.with(|a| {
                let mut anims = a.borrow_mut();
                anims.iter_mut().for_each(|a| a.mark_for_removal());
            });
        }

        if let Some(entity) = active_entity {
            AI.with(|ai| {
                let mut ai = ai.borrow_mut();
                ai.update(entity);
            });
        }
    }

    pub fn draw_graphics_mode(renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                              scale_x: f32, scale_y: f32, millis: u32) {
        ANIMATIONS.with(|a| {
            let anims = a.borrow();

            for anim in anims.iter() {
                anim.draw_graphics_mode(renderer, offset_x, offset_y, scale_x, scale_y, millis);
            }
        })
    }

    pub fn has_active_animations(entity: &Rc<RefCell<EntityState>>) -> bool {
        ANIMATIONS.with(|a| {
            let anims = a.borrow();

            for anim in anims.iter() {
                if *anim.get_owner().borrow() == *entity.borrow() {
                    return true;
                }
            }
            false
        })
    }

    pub fn add_animation(anim: Box<Animation>) {
        ANIMS_TO_ADD.with(|a| {
            let mut anims = a.borrow_mut();

            anims.push(anim);
        });
    }

    /// Returns true if the game is currently in turn mode, false otherwise
    pub fn is_in_turn_mode() -> bool {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        area_state.turn_timer.is_active()
    }

    /// Returns true if the PC has the current turn, false otherwise
    pub fn is_pc_current() -> bool {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        if let Some(entity) = area_state.turn_timer.current() {
            return entity.borrow().is_party_member();
        }
        false
    }

    pub fn is_current(entity: &Rc<RefCell<EntityState>>) -> bool {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        if let Some(ref current) = area_state.turn_timer.current() {
            return Rc::ptr_eq(current, entity);
        }
        false
    }

    fn get_target(entity: &Rc<RefCell<EntityState>>,
                  target: &Rc<RefCell<EntityState>>) -> (f32, f32, f32) {
        let (target_x, target_y) = {
            let target = target.borrow();
            (target.location.x as f32 + target.size.width as f32 / 2.0 - 0.5,
             target.location.y as f32 + target.size.height as f32/ 2.0 - 0.5)
        };

        let sizes = (entity.borrow().size.diagonal + target.borrow().size.diagonal) / 2.0;
        let mut range = sizes + entity.borrow().actor.stats.attack_distance();

        let area = GameState::area_state();
        let vis_dist = area.borrow().area.vis_dist as f32;
        if range > vis_dist {
            range = vis_dist;
        }

        trace!("Getting move target at {}, {} within {}", target_x, target_y, range);
        (target_x, target_y, range)
    }

    pub fn can_move_towards(entity: &Rc<RefCell<EntityState>>,
                            target: &Rc<RefCell<EntityState>>) -> bool {
        let (x, y, dist) = GameState::get_target(entity, target);
        GameState::can_move_towards_point(entity, Vec::new(), x, y, dist)
    }

    pub fn move_towards(entity: &Rc<RefCell<EntityState>>,
                        target: &Rc<RefCell<EntityState>>) -> bool {
        let (x, y, dist) = GameState::get_target(entity, target);
        GameState::move_towards_point(entity, Vec::new(), x, y, dist, None)
    }

    pub fn can_move_to(entity: &Rc<RefCell<EntityState>>, x: i32, y: i32) -> bool {
        GameState::can_move_towards_point(entity, Vec::new(), x as f32, y as f32, 0.6)
    }

    pub fn move_to(entity: &Rc<RefCell<EntityState>>, x: i32, y: i32) -> bool {
        GameState::move_towards_point(entity, Vec::new(), x as f32, y as f32, 0.6, None)
    }

    pub fn move_towards_point(entity: &Rc<RefCell<EntityState>>, entities_to_ignore: Vec<usize>,
                              x: f32, y: f32, dist: f32, cb: Option<Box<ScriptCallback>>) -> bool {
        let anim = STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            debug!("Moving '{}' to {},{}", entity.borrow().actor.actor.name, x, y);

            let start_time = time::Instant::now();
            let path = {
                let area_state = state.area_state.borrow();
                match state.path_finder.find(&area_state, entity.borrow(),
                                             entities_to_ignore, x, y, dist) {
                    None => return None,
                    Some(path) => path,
                }
            };
            debug!("Path finding complete in {} secs",
                  util::format_elapsed_secs(start_time.elapsed()));

            let entity = Rc::clone(entity);
            let mut anim = MoveAnimation::new(entity, path, CONFIG.display.animation_base_time_millis);
            anim.set_callback(cb);
            Some(anim)
        });

        match anim {
            None => false,
            Some(anim) => {
                ANIMATIONS.with(|a| {
                    let mut anims = a.borrow_mut();
                    for anim in anims.iter_mut() {
                        if Rc::ptr_eq(entity, anim.get_owner()) {
                            anim.mark_for_removal();
                        }
                    }
                    anims.push(Box::new(anim));
                });
                true
            }
        }
    }

    pub fn can_move_towards_point(entity: &Rc<RefCell<EntityState>>, entities_to_ignore: Vec<usize>,
                                  x: f32, y: f32, dist: f32) -> bool {
        if entity.borrow().actor.ap() < Module::rules().movement_ap {
            return false;
        }

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            let area_state = state.area_state.borrow();

            let start_time = time::Instant::now();
            let val = match state.path_finder.find(&area_state, entity.borrow(),
                                                   entities_to_ignore, x, y, dist) {
                None => false,
                Some(_) => true,
            };
            debug!("Path finding complete in {} secs",
                  util::format_elapsed_secs(start_time.elapsed()));

            val
        })
    }
}
