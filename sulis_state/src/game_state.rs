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

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::time;

use sulis_core::config::Config;
use sulis_core::io::GraphicsRenderer;
use sulis_core::util::{self, invalid_data_error, ExtInt, Point};
use sulis_module::on_trigger::QuestEntryState;
use sulis_module::{
    area::{Trigger, TriggerKind, ToKind},
    Actor, Module, ObjectSize, OnTrigger, Time,
};

use crate::animation::{self, particle_generator::Param, Anim, AnimSaveState, AnimState};
use crate::script::{script_cache, script_callback, Script, ScriptCallback, ScriptEntity};
use crate::{
    AreaState, ChangeListener, ChangeListenerList, Effect, EntityState, Formation,
    ItemList, ItemState, Location, PartyStash, PathFinder, QuestStateSet,
    SaveState, TurnManager, UICallback, WorldMapState, AI, MOVE_TO_THRESHOLD,
};

thread_local! {
    static TURN_MANAGER: Rc<RefCell<TurnManager>> = Rc::new(RefCell::new(TurnManager::default()));
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
    static AI: RefCell<AI> = RefCell::new(AI::new());
    static CLEAR_ANIMS: Cell<bool> = Cell::new(false);
    static MODAL_LOCKED: Cell<bool> = Cell::new(false);
    static ANIMATIONS: RefCell<AnimState> = RefCell::new(AnimState::new());
    static ANIMS_TO_ADD: RefCell<Vec<Anim>> = RefCell::new(Vec::new());
}

pub struct GameState {
    areas: HashMap<String, Rc<RefCell<AreaState>>>,
    area_state: Rc<RefCell<AreaState>>,
    world_map: WorldMapState,
    quests: QuestStateSet,
    selected: Vec<Rc<RefCell<EntityState>>>,
    user_zoom: f32,
    party: Vec<Rc<RefCell<EntityState>>>,
    party_formation: Rc<RefCell<Formation>>,
    party_coins: i32,
    party_stash: Rc<RefCell<PartyStash>>,

    // listener returns the first selected party member
    party_listeners: ChangeListenerList<Option<Rc<RefCell<EntityState>>>>,
    party_death_listeners: ChangeListenerList<Vec<Rc<RefCell<EntityState>>>>,
    path_finder: PathFinder,
    ui_callbacks: Vec<UICallback>,
}

const MIN_ZOOM: f32 = 0.7;
const MAX_ZOOM: f32 = 2.0;

impl GameState {
    pub fn load(save_state: SaveState) -> Result<(), Error> {
        TURN_MANAGER.with(|mgr| {
            mgr.borrow_mut().load(save_state.total_elapsed_millis);
        });
        ANIMATIONS.with(|anims| anims.borrow_mut().clear());
        STATE.with(|state| *state.borrow_mut() = None);
        CLEAR_ANIMS.with(|c| c.set(false));
        MODAL_LOCKED.with(|c| c.set(false));
        ANIMS_TO_ADD.with(|anims| anims.borrow_mut().clear());
        AI.with(|ai| *ai.borrow_mut() = AI::new());
        script_cache::setup().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;

        let game_state: Result<GameState, Error> = {
            let mut areas = HashMap::new();
            for (id, area_save) in save_state.areas {
                let area_state = AreaState::load(&id, area_save)?;

                areas.insert(id, Rc::new(RefCell::new(area_state)));
            }

            let area_state = match areas.get(&save_state.current_area) {
                Some(ref area) => Ok(Rc::clone(area)),
                None => invalid_data_error(&format!(
                    "Unable to load current area '{}'",
                    save_state.current_area
                )),
            }?;

            let path_finder = PathFinder::new(&area_state.borrow().area.area);

            let mut entities = HashMap::new();
            let mut selected = Vec::new();
            let mut party = Vec::new();

            for entity_save in save_state.manager.entities {
                let index = entity_save.index;
                let entity = Rc::new(RefCell::new(EntityState::load(entity_save, &areas)?));
                entities.insert(index, entity);
            }

            for index in save_state.party {
                match entities.get(&index) {
                    None => return invalid_data_error(&format!("Invalid party index {}", index)),
                    Some(ref entity) => party.push(Rc::clone(entity)),
                }
            }

            for index in save_state.selected {
                match entities.get(&index) {
                    None => return invalid_data_error(&format!("Invalid selected index {}", index)),
                    Some(ref entity) => selected.push(Rc::clone(entity)),
                }
            }

            for entity in entities.values() {
                let area_state = match areas.get(&entity.borrow().location.area_id) {
                    Some(state) => state,
                    None => unreachable!(),
                };

                let is_dead = entity.borrow().actor.is_dead();
                let location = entity.borrow().location.clone();
                area_state
                    .borrow_mut()
                    .load_entity(entity, location, is_dead)?;
            }

            let mut effects = HashMap::new();

            let mgr = GameState::turn_manager();
            mgr.borrow_mut().cur_ai_group_index = save_state.manager.cur_ai_group_index;
            for (key, value) in save_state.manager.ai_groups {
                let index = match key.parse::<usize>() {
                    Ok(val) => val,
                    Err(e) => {
                        let err = Error::new(ErrorKind::InvalidInput, e);
                        return Err(err);
                    }
                };
                mgr.borrow_mut().ai_groups.insert(index, value);
            }

            for effect_save in save_state.manager.effects {
                let old_index = effect_save.index;
                let new_index = mgr.borrow().get_next_effect_index();

                let mut effect = Effect::load(effect_save, new_index, &entities)?;
                if let Some(index) = effect.entity {
                    let entity = match entities.get(&index) {
                        None => {
                            return invalid_data_error(&format!("Invalid effect entity {}", index));
                        }
                        Some(ref entity) => Rc::clone(entity),
                    };

                    // the index has changed with the load
                    effect.entity = Some(entity.borrow().index());

                    let new_idx =
                        mgr.borrow_mut()
                            .add_effect(effect, &entity, Vec::new(), Vec::new());
                    assert!(new_index == new_idx);
                    effects.insert(old_index, new_index);
                    continue;
                }

                if let Some(surface) = effect.surface.clone() {
                    let area = match areas.get(&surface.area_id) {
                        None => {
                            return invalid_data_error(&format!(
                                "Invalid area ID '{}'",
                                surface.area_id
                            ));
                        }
                        Some(area) => area,
                    };

                    let new_idx = mgr.borrow_mut().add_surface(
                        effect,
                        area,
                        surface.points,
                        Vec::new(),
                        Vec::new(),
                    );
                    assert!(new_index == new_idx);
                    effects.insert(old_index, new_index);
                }
            }

            let mut marked = HashMap::new();
            for anim in save_state.anims {
                match anim.load(&entities, &effects, &mut marked) {
                    None => (),
                    Some(anim) => GameState::add_animation(anim),
                }
            }

            let mgr = GameState::turn_manager();
            for (index, vec) in marked {
                mgr.borrow_mut().add_removal_listener_for_effect(index, vec);
            }

            let formation = save_state.formation;

            let party_coins = save_state.coins;

            let mut stash = ItemList::new();
            for item_save in save_state.stash {
                let item = &item_save.item;
                let item = match Module::create_get_item(&item.id, &item.adjectives) {
                    None => invalid_data_error(&format!("No item with ID '{}'", item_save.item.id)),
                    Some(item) => Ok(item),
                }?;

                stash.add_quantity(item_save.quantity, ItemState::new(item));
            }

            let quests = QuestStateSet::load(save_state.quests);
            let mut world_map = save_state.world_map;
            world_map.load();

            Ok(GameState {
                areas,
                area_state,
                path_finder,
                party,
                selected,
                user_zoom: save_state.zoom,
                party_formation: Rc::new(RefCell::new(formation)),
                party_coins,
                party_stash: Rc::new(RefCell::new(PartyStash::new(stash))),
                party_listeners: ChangeListenerList::default(),
                party_death_listeners: ChangeListenerList::default(),
                ui_callbacks: Vec::new(),
                world_map,
                quests,
            })
        };

        let game_state = game_state?;
        STATE.with(|state| {
            *state.borrow_mut() = Some(game_state);
        });

        let pc = GameState::player();
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        area_state.update_view_visibility();
        area_state.push_scroll_to_callback(pc);

        Ok(())
    }

    pub fn init(pc_actor: Rc<Actor>) -> Result<(), Error> {
        TURN_MANAGER.with(|mgr| {
            let rules = Module::rules();
            let starting_time = Module::campaign().starting_time;
            mgr.borrow_mut().load(rules.compute_millis(starting_time));
        });

        script_cache::setup().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
        let game_state = GameState::new(pc_actor)?;
        STATE.with(|state| {
            *state.borrow_mut() = Some(game_state);
        });

        let pc = GameState::player();
        let area_state = GameState::area_state();
        area_state.borrow_mut().update_view_visibility();
        area_state
            .borrow_mut()
            .push_scroll_to_callback(Rc::clone(&pc));
        area_state.borrow_mut().on_load_fired = true;
        let area_state = area_state.borrow();
        GameState::add_ui_callbacks_of_kind(
            &area_state.area.area.triggers,
            TriggerKind::OnCampaignStart,
            &pc,
            &pc,
        );
        GameState::add_ui_callbacks_of_kind(
            &area_state.area.area.triggers,
            TriggerKind::OnAreaLoad,
            &pc,
            &pc,
        );

        Ok(())
    }

    fn new(pc: Rc<Actor>) -> Result<GameState, Error> {
        let party_coins = pc.inventory.pc_starting_coins();
        let mut party_stash = ItemList::new();
        for (qty, item) in pc.inventory.pc_starting_item_iter() {
            let item_state = ItemState::new(item);
            party_stash.add_quantity(qty, item_state);
        }

        let campaign = Module::campaign();

        let area_state = GameState::setup_area_state(&campaign.starting_area)?;

        debug!(
            "Setting up PC {}, with {:?}",
            &pc.name, &campaign.starting_location
        );
        let location = Location::from_point(&campaign.starting_location,
                                            &area_state.borrow().area.area);

        if !location.coords_valid(location.x, location.y) {
            error!("Starting location coordinates must be valid for the starting area.");
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Unable to create starting location.",
            ));
        }

        let index = match area_state
            .borrow_mut()
            .add_actor(pc, location, None, true, None)
        {
            Err(_) => {
                error!("Player character starting location must be within bounds and passable.");
                return invalid_data_error("Unable to add player character at starting location");
            }
            Ok(index) => index,
        };

        let mgr = GameState::turn_manager();
        let pc_state = mgr.borrow().entity(index);

        pc_state.borrow_mut().actor.init_turn();

        let path_finder = PathFinder::new(&area_state.borrow().area.area);

        let mut areas: HashMap<String, Rc<RefCell<AreaState>>> = HashMap::new();
        areas.insert(campaign.starting_area.to_string(), Rc::clone(&area_state));

        let mut party = Vec::new();
        party.push(Rc::clone(&pc_state));

        let mut selected = Vec::new();
        selected.push(Rc::clone(&pc_state));

        Ok(GameState {
            user_zoom: 1.0,
            areas,
            area_state,
            path_finder,
            selected,
            party,
            party_formation: Rc::new(RefCell::new(Formation::default())),
            party_coins,
            party_stash: Rc::new(RefCell::new(PartyStash::new(party_stash))),
            party_listeners: ChangeListenerList::default(),
            party_death_listeners: ChangeListenerList::default(),
            ui_callbacks: Vec::new(),
            world_map: WorldMapState::new(),
            quests: QuestStateSet::new(),
        })
    }

    pub fn set_world_map_location_visible(location: &str, visible: bool) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.world_map.set_visible(location, visible);
        })
    }

    pub fn set_world_map_location_enabled(location: &str, enabled: bool) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.world_map.set_enabled(location, enabled);
        })
    }

    pub fn world_map() -> WorldMapState {
        STATE.with(|state| {
            let state = state.borrow();
            let state = state.as_ref().unwrap();

            state.world_map.clone()
        })
    }

    pub fn quest_state() -> QuestStateSet {
        STATE.with(|state| {
            let state = state.borrow();
            let state = state.as_ref().unwrap();

            state.quests.clone()
        })
    }

    pub fn add_quest_state_change_listener(listener: ChangeListener<QuestStateSet>) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.quests.listeners.add(listener);
        })
    }

    pub fn get_quest_state(quest: String) -> QuestEntryState {
        STATE.with(|state| {
            let state = state.borrow();
            let state = state.as_ref().unwrap();
            state.quests.state(&quest)
        })
    }

    pub fn get_quest_entry_state(quest: String, entry: String) -> QuestEntryState {
        STATE.with(|state| {
            let state = state.borrow();
            let state = state.as_ref().unwrap();
            state.quests.entry_state(&quest, &entry)
        })
    }

    pub fn set_quest_state(quest: String, entry_state: QuestEntryState) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();
            state.quests.set_state(&quest, entry_state);
        });
    }

    pub fn set_quest_entry_state(quest: String, entry: String, entry_state: QuestEntryState) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();
            state.quests.set_entry_state(&quest, &entry, entry_state);
        })
    }

    pub fn set_user_zoom(mut zoom: f32) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            if zoom > MAX_ZOOM {
                zoom = MAX_ZOOM;
            } else if zoom < MIN_ZOOM {
                zoom = MIN_ZOOM;
            }

            state.user_zoom = zoom;
        });
    }

    pub fn user_zoom() -> f32 {
        STATE.with(|state| state.borrow().as_ref().unwrap().user_zoom)
    }

    pub fn turn_manager() -> Rc<RefCell<TurnManager>> {
        TURN_MANAGER.with(|m| Rc::clone(&m))
    }

    pub fn set_selected_party_member(entity: Rc<RefCell<EntityState>>) {
        GameState::select_party_members(vec![entity]);
    }

    pub fn clear_selected_party_member() {
        GameState::select_party_members(Vec::new());
    }

    pub fn select_party_members(mut members: Vec<Rc<RefCell<EntityState>>>) {
        for member in members.iter() {
            if !member.borrow().is_party_member() {
                warn!(
                    "Attempted to select non-party member {}",
                    member.borrow().actor.actor.id
                );
            }
        }

        members.retain(|e| !e.borrow().actor.is_dead());

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            state.selected.clear();
            // add in party member order
            for party_member in state.party.iter() {
                for member in members.iter() {
                    if Rc::ptr_eq(party_member, member) {
                        state.selected.push(Rc::clone(member));
                        // GameState::create_selection_animation(&member);
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

    pub fn create_damage_animation(entity: &Rc<RefCell<EntityState>>) {
        let time = 200;
        let time_f32 = time as f32 / 1000.0;
        let duration = ExtInt::Int(time);
        let color = [
            Param::with_jerk(0.0, 1.0 / time_f32, 0.0, -1.0 / time_f32),
            Param::fixed(1.0),
            Param::fixed(1.0),
            Param::fixed(1.0),
        ];
        let color_sec = [
            Param::with_jerk(0.0, 0.7 / time_f32, 0.0, -0.7 / time_f32),
            Param::fixed(0.0),
            Param::fixed(0.0),
            Param::fixed(0.0),
        ];
        let anim = Anim::new_entity_color(entity, duration, color, color_sec);
        GameState::add_animation(anim);
    }

    pub fn selected() -> Vec<Rc<RefCell<EntityState>>> {
        STATE.with(|s| s.borrow().as_ref().unwrap().selected.clone())
    }

    pub fn remove_party_member(entity: Rc<RefCell<EntityState>>) {
        info!("Remove party member {}", entity.borrow().actor.actor.id);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            entity.borrow_mut().remove_from_party();
            state.party.retain(|e| !Rc::ptr_eq(e, &entity));

            state.selected.retain(|e| !Rc::ptr_eq(e, &entity));

            let entity = match state.selected.first() {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            };
            state.party_listeners.notify(&entity);
        });

        let area_state = GameState::area_state();
        area_state.borrow_mut().update_view_visibility();
        area_state.borrow_mut().pc_vis_full_redraw();
    }

    fn add_disabled_party_members() {
        for member in GameState::party() {
            {
                let mut member = member.borrow_mut();
                if member.actor.is_dead() && member.actor.is_disabled() {
                    member.actor.set_disabled(false);
                    member.actor.add_hp(1);
                } else {
                    continue;
                }
            }

            let location = member.borrow().location.clone();
            let area_state = GameState::get_area_state(&location.area_id).unwrap();
            let index = member.borrow().index();
            let mut area_state = area_state.borrow_mut();
            area_state.remove_matching_prop(
                location.x,
                location.y,
                &member.borrow().actor.actor.name,
            );
            match area_state.transition_entity_to(&member, index, location) {
                Err(e) => {
                    warn!("Error re-adding disabled party member:");
                    warn!("{}", e);
                }
                Ok(_) => (),
            }
            let mgr = GameState::turn_manager();
            mgr.borrow_mut().readd_entity(&member);

            let anim = Anim::new_entity_recover(&member);
            GameState::add_animation(anim);
        }
    }

    fn remove_disabled_party_members() -> bool {
        let mut notify = false;
        for member in GameState::party().iter() {
            {
                let member = member.borrow();
                if !member.actor.is_dead() {
                    continue;
                }
                if member.actor.is_disabled() {
                    continue;
                }
            }

            let script = &Module::campaign().on_party_death_script;
            Script::trigger(&script.id, &script.func, ScriptEntity::from(member));

            {
                let member = member.borrow();
                let prop = match &member.actor.actor.race.pc_death_prop {
                    None => continue,
                    Some(ref prop) => Rc::clone(prop),
                };

                let area_state = GameState::get_area_state(&member.location.area_id).unwrap();
                let x = member.location.x;
                let y = member.location.y;
                let name = member.actor.actor.name.to_string();
                let enabled = member.actor.is_disabled();
                area_state
                    .borrow_mut()
                    .add_prop_at(&prop, x, y, enabled, Some(name));
            }
            notify = true;
        }

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            let pc = Rc::clone(&state.party[0]); // don't ever remove the PC
            state.party.retain(|e| {
                if Rc::ptr_eq(&e, &pc) {
                    return true;
                }
                let actor = &e.borrow().actor;
                !actor.is_dead() || actor.is_disabled()
            });
            state.selected.retain(|e| !e.borrow().actor.is_dead());

            if notify {
                info!("Removed or Disabled a dead party member; notifying listeners");
                state.party_death_listeners.notify(&state.party);

                let entity = match state.selected.first() {
                    None => None,
                    Some(ref entity) => Some(Rc::clone(entity)),
                };
                state.party_listeners.notify(&entity);
                true
            } else {
                false
            }
        })
    }

    pub fn handle_disabled_party_members() {
        let update = GameState::remove_disabled_party_members();

        if !GameState::is_combat_active() {
            GameState::add_disabled_party_members();
        }

        if update {
            let area_state = GameState::area_state();
            area_state.borrow_mut().update_view_visibility();
            area_state.borrow_mut().pc_vis_full_redraw();
        }
    }

    pub fn has_party_member(id: &str) -> bool {
        for entity in GameState::party() {
            if &entity.borrow().actor.actor.id == id {
                return true;
            }
        }

        false
    }

    pub fn add_party_member(entity: Rc<RefCell<EntityState>>, show_portrait: bool) {
        info!("Add party member {}", entity.borrow().actor.actor.id);
        let mgr = GameState::turn_manager();
        if !mgr.borrow().is_combat_active() {
            entity.borrow_mut().actor.init_turn();
        }

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            entity.borrow_mut().add_to_party(show_portrait);
            state
                .area_state
                .borrow_mut()
                .compute_pc_visibility(&entity, 0, 0);
            state.party.push(Rc::clone(&entity));

            let entity = match state.selected.first() {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            };
            state.party_listeners.notify(&entity);
        });

        let area_state = GameState::area_state();
        area_state.borrow_mut().update_view_visibility();
    }

    pub fn add_party_death_listener(listener: ChangeListener<Vec<Rc<RefCell<EntityState>>>>) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();
            state.party_death_listeners.add(listener);
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

    fn find_link(state: &AreaState, from: &str) -> Option<Point> {
        for transition in state.area.transitions.iter() {
            match transition.to {
                ToKind::Area { ref id, .. } | ToKind::FindLink { ref id, .. } => {
                    if id == from {
                        return Some(transition.from);
                    }
                },
                _ => (),
            }
        }

        None
    }

    fn preload_area(area_id: &str) -> Result<(), Error> {
        if let Some(_) = GameState::get_area_state(area_id) { return Ok(()); }

        let area_state = GameState::setup_area_state(area_id)?;

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();
            state.areas.insert(area_id.to_string(), area_state);
        });

        Ok(())
    }


    pub fn transition_to(area_id: Option<&str>,
                         p: Option<Point>,
                         offset: Point,
                         travel_time: Time) {
        info!("Area transition to {:?} at {:?}", area_id, p);

        // perform area preload if area is not already loaded
        if let Some(id) = area_id {
            if let Err(e) = GameState::preload_area(id) {
                error!("Error loading {} for transition", id);
                error!("{}", e);
                return;
            }
        }

        let (area, location) = match area_id {
            None => {
                // intra-area transition
                if let None = p {
                    error!("No point specified for intra area transition");
                    return;
                }

                (GameState::area_state(), p.unwrap())
            },
            Some(id) => {
                // transition to another area
                let state = GameState::get_area_state(id).unwrap();

                if Rc::ptr_eq(&state, &GameState::area_state()) {
                    error!("Area transition to already loaded area, but not intra area: {}",
                            state.borrow().area.area.id);
                    return;
                }

                // use location if specified, otherwise find appropriate link
                let location = match p {
                    Some(p) => p,
                    None => {
                        let old = GameState::area_state();
                        let old = old.borrow();
                        match GameState::find_link(&state.borrow(), &old.area.area.id) {
                            None => {
                                error!("Error finding linked coordinates for transition {}", id);
                                return;
                            }, Some(loc) => loc,
                        }
                    }
                };
                (state, location)
            }
        };

        if !GameState::check_location(&location, &area) {
            error!("Invalid transition location to {:?} in {}",
                   location, area.borrow().area.area.id);
            return;
        }

        // now set the new area as the current area if it is not already
        if !Rc::ptr_eq(&GameState::area_state(), &area) {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let state = state.as_mut().unwrap();
                let path_finder = PathFinder::new(&area.borrow().area.area);
                state.path_finder = path_finder;
                state.area_state = area;
            });
        }

        let (x, y) = (location.x + offset.x, location.y + offset.y);

        // clean up animations and surfaces
        GameState::set_clear_anims();

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();

            let mgr = GameState::turn_manager();
            {
                for entity in state.party.iter() {
                    let entity_index = entity.borrow().index();
                    let area_id = entity.borrow().location.area_id.to_string();
                    let area = &state.areas.get(&area_id).unwrap();
                    let surfaces = area.borrow_mut().remove_entity(&entity, &mgr.borrow());

                    for surface in surfaces {
                        mgr.borrow_mut().remove_from_surface(entity_index, surface);
                    }
                }
            }
        });

        let pc = GameState::player();
        let mgr = GameState::turn_manager();
        mgr.borrow_mut().add_time(travel_time);

        // find transition locations for all party members
        let area_state = GameState::area_state();
        let base_location = Location::new(x, y, &area_state.borrow().area.area);
        for entity in GameState::party() {
            entity.borrow_mut().clear_pc_vis();
            let mut cur_location = base_location.clone();
            GameState::find_transition_location(
                &mut cur_location,
                &entity.borrow().size,
                &area_state.borrow(),
            );
            info!(
                "Transitioning {} to {},{}",
                entity.borrow().actor.actor.name,
                cur_location.x,
                cur_location.y
            );
            let index = entity.borrow().index();

            match area_state
                .borrow_mut()
                .transition_entity_to(&entity, index, cur_location)
            {
                Ok(_) => (),
                Err(e) => {
                    warn!("Unable to add party member");
                    warn!("{}", e);
                }
            }
        }

        area_state
            .borrow_mut()
            .push_scroll_to_callback(Rc::clone(&pc));

        let mut area_state = area_state.borrow_mut();
        area_state.update_view_visibility();
        if !area_state.on_load_fired {
            area_state.on_load_fired = true;
            GameState::add_ui_callbacks_of_kind(
                &area_state.area.area.triggers,
                TriggerKind::OnAreaLoad,
                &pc,
                &pc,
            );
        }
    }

    fn find_transition_location(
        location: &mut Location,
        size: &Rc<ObjectSize>,
        area_state: &AreaState,
    ) {
        let (base_x, base_y) = (location.x, location.y);
        let mut search_size = 0;
        while search_size < 10 {
            // TODO this does a lot of unneccesary checking
            for y in -search_size..search_size + 1 {
                for x in -search_size..search_size + 1 {
                    if area_state.is_passable_size(size, base_x + x, base_y + y) {
                        location.x = base_x + x;
                        location.y = base_y + y;
                        return;
                    }
                }
            }

            search_size += 1;
        }

        warn!("Unable to find transition locations for all party members");
    }

    fn check_location(p: &Point, area_state: &Rc<RefCell<AreaState>>) -> bool {
        let location = Location::from_point(p, &area_state.borrow().area.area);
        if !location.coords_valid(location.x, location.y) {
            error!(
                "Location coordinates {},{} are not valid for area {}",
                location.x, location.y, location.area_id
            );
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

        let state = AreaState::new(area, None)?;
        let area_state = Rc::new(RefCell::new(state));
        area_state.borrow_mut().populate();

        Ok(area_state)
    }

    pub fn add_ui_callback(
        cb: Vec<OnTrigger>,
        parent: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) {
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

    pub fn add_ui_callbacks_of_kind(
        callbacks: &Vec<Trigger>,
        kind: TriggerKind,
        parent: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) {
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

    pub fn is_modal_locked() -> bool {
        MODAL_LOCKED.with(|c| c.get())
    }

    pub fn set_modal_locked(locked: bool) {
        MODAL_LOCKED.with(|c| c.set(locked))
    }

    fn check_clear_anims() -> bool {
        CLEAR_ANIMS.with(|c| c.replace(false))
    }

    pub fn set_clear_anims() {
        CLEAR_ANIMS.with(|c| c.set(true));
    }

    pub fn area_state_ids() -> Vec<String> {
        STATE.with(|s| {
            s.borrow()
                .as_ref()
                .unwrap()
                .areas
                .keys()
                .map(|k| k.to_string())
                .collect()
        })
    }

    pub fn get_area_state(id: &str) -> Option<Rc<RefCell<AreaState>>> {
        STATE.with(|s| match s.borrow().as_ref().unwrap().areas.get(id) {
            None => None,
            Some(area_state) => Some(Rc::clone(&area_state)),
        })
    }

    pub fn area_state() -> Rc<RefCell<AreaState>> {
        STATE.with(|s| Rc::clone(&s.borrow().as_ref().unwrap().area_state))
    }

    #[must_use]
    pub fn update(millis: u32) -> Option<UICallback> {
        let ui_cb = STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            state.ui_callbacks.pop()
        });

        let to_add: Vec<Anim> = ANIMS_TO_ADD.with(|a| {
            let mut anims = a.borrow_mut();

            let to_add = anims.drain(0..).collect();

            to_add
        });

        let (update_cbs, complete_cbs) = ANIMATIONS.with(|a| a.borrow_mut().update(to_add, millis));
        update_cbs.into_iter().for_each(|cb| cb.on_anim_update());
        complete_cbs
            .into_iter()
            .for_each(|cb| cb.on_anim_complete());

        let mgr = GameState::turn_manager();
        let (turn_cbs, removal_cbs) = mgr.borrow_mut().update(millis);

        script_callback::fire_round_elapsed(turn_cbs);
        script_callback::fire_on_removed(removal_cbs);

        let cbs = mgr.borrow_mut().update_entity_move_callbacks();
        script_callback::fire_on_moved(cbs);

        let on_moved_cbs = mgr.borrow_mut().update_on_moved_in_surface();
        script_callback::fire_on_moved_in_surface(on_moved_cbs);

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();

            let mut area_state = state.area_state.borrow_mut();
            area_state.update();
        });

        if GameState::check_clear_anims() {
            ANIMATIONS.with(|a| a.borrow_mut().clear_all_blocking_anims());
        }

        let current = mgr.borrow().current();
        if let Some(entity) = current {
            AI.with(|ai| {
                let mut ai = ai.borrow_mut();
                ai.update(entity);
            });
        }

        GameState::handle_disabled_party_members();

        let campaign = Module::campaign();
        if let Some(script_data) = &campaign.on_tick_script {
            script_cache::set_report_enabled(false);
            let arg: Option<bool> = None;
            Script::trigger(&script_data.id, &script_data.func, arg);
            script_cache::set_report_enabled(true);
        }

        ui_cb
    }

    pub fn draw_above_entities(
        renderer: &mut GraphicsRenderer,
        offset_x: f32,
        offset_y: f32,
        scale_x: f32,
        scale_y: f32,
        millis: u32,
    ) {
        ANIMATIONS.with(|a| {
            a.borrow()
                .draw_above_entities(renderer, offset_x, offset_y, scale_x, scale_y, millis)
        });
    }

    pub fn draw_below_entities(
        renderer: &mut GraphicsRenderer,
        offset_x: f32,
        offset_y: f32,
        scale_x: f32,
        scale_y: f32,
        millis: u32,
    ) {
        ANIMATIONS.with(|a| {
            a.borrow()
                .draw_below_entities(renderer, offset_x, offset_y, scale_x, scale_y, millis)
        });
    }

    pub fn has_any_blocking_animations() -> bool {
        ANIMATIONS.with(|a| a.borrow().has_any_blocking_anims())
    }

    pub fn has_blocking_animations(entity: &Rc<RefCell<EntityState>>) -> bool {
        ANIMATIONS.with(|a| a.borrow().has_blocking_anims(entity))
    }

    pub fn remove_blocking_animations(entity: &Rc<RefCell<EntityState>>) {
        ANIMATIONS.with(|a| a.borrow_mut().clear_blocking_anims(entity));
    }

    pub fn remove_all_blocking_animations() {
        ANIMATIONS.with(|a| a.borrow_mut().clear_all_blocking_anims());
    }

    pub fn add_animation(anim: Anim) {
        ANIMS_TO_ADD.with(|a| {
            let mut anims = a.borrow_mut();

            anims.push(anim);
        });
    }

    pub fn save_anims() -> Vec<AnimSaveState> {
        ANIMATIONS.with(|a| a.borrow().save_anims())
    }

    /// Returns true if the game is currently in turn mode, false otherwise
    pub fn is_combat_active() -> bool {
        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        mgr.is_combat_active()
    }

    /// Returns true if the PC has the current turn, false otherwise
    pub fn is_pc_current() -> bool {
        let mgr = GameState::turn_manager();
        if let Some(entity) = mgr.borrow().current() {
            return entity.borrow().is_party_member();
        }

        false
    }

    pub fn is_current(entity: &Rc<RefCell<EntityState>>) -> bool {
        let mgr = GameState::turn_manager();
        if let Some(current) = mgr.borrow().current() {
            return Rc::ptr_eq(&current, entity);
        }
        false
    }

    fn get_target(
        entity: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) -> (f32, f32, f32) {
        let (target_x, target_y) = {
            let target = target.borrow();
            (
                target.location.x as f32 + (target.size.width / 2) as f32,
                target.location.y as f32 + (target.size.height / 2) as f32,
            )
        };

        let sizes = (entity.borrow().size.diagonal + target.borrow().size.diagonal) / 2.0;
        let mut range = sizes + entity.borrow().actor.stats.attack_distance();

        let area = GameState::area_state();
        let vis_dist = area.borrow().area.area.vis_dist as f32;
        if range > vis_dist {
            range = vis_dist;
        }

        trace!(
            "Getting move target at {}, {} within {}",
            target_x,
            target_y,
            range
        );
        (target_x, target_y, range)
    }

    pub fn can_move_towards(
        entity: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) -> bool {
        let (x, y, dist) = GameState::get_target(entity, target);
        GameState::can_move_towards_point(entity, Vec::new(), x, y, dist)
    }

    pub fn move_towards(
        entity: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) -> bool {
        let (x, y, dist) = GameState::get_target(entity, target);
        GameState::move_towards_point(entity, Vec::new(), x, y, dist, None)
    }

    pub fn can_move_to(entity: &Rc<RefCell<EntityState>>, x: i32, y: i32) -> bool {
        GameState::can_move_towards_point(entity, Vec::new(), x as f32, y as f32, MOVE_TO_THRESHOLD)
    }

    pub fn move_to(entity: &Rc<RefCell<EntityState>>, x: i32, y: i32) -> bool {
        GameState::move_towards_point(
            entity,
            Vec::new(),
            x as f32,
            y as f32,
            MOVE_TO_THRESHOLD,
            None,
        )
    }

    pub fn move_towards_point(
        entity: &Rc<RefCell<EntityState>>,
        entities_to_ignore: Vec<usize>,
        x: f32,
        y: f32,
        dist: f32,
        cb: Option<Box<ScriptCallback>>,
    ) -> bool {
        if entity.borrow().actor.stats.move_disabled {
            return false;
        }

        // if entity cannot move even 1 square
        if entity.borrow().actor.ap() < entity.borrow().actor.get_move_ap_cost(1) {
            return false;
        }

        let anim = STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            debug!(
                "Moving '{}' to {},{}",
                entity.borrow().actor.actor.name,
                x,
                y
            );

            let start_time = time::Instant::now();
            let path = {
                let area_state = state.area_state.borrow();
                match state.path_finder.find(
                    &area_state,
                    entity.borrow(),
                    entities_to_ignore,
                    true,
                    x,
                    y,
                    dist,
                ) {
                    None => return None,
                    Some(path) => path,
                }
            };
            debug!(
                "Path finding complete in {} secs",
                util::format_elapsed_secs(start_time.elapsed())
            );

            let mut anim =
                animation::move_animation::new(entity, path, Config::animation_base_time_millis());
            if let Some(cb) = cb {
                anim.add_completion_callback(cb);
            }
            Some(anim)
        });

        match anim {
            None => false,
            Some(anim) => {
                GameState::remove_blocking_animations(entity);
                GameState::add_animation(anim);
                true
            }
        }
    }

    pub fn can_move_towards_point(
        entity: &Rc<RefCell<EntityState>>,
        entities_to_ignore: Vec<usize>,
        x: f32,
        y: f32,
        dist: f32,
    ) -> bool {
        if entity.borrow().actor.stats.move_disabled {
            return false;
        }

        // if entity cannot move even 1 square
        if entity.borrow().actor.ap() < entity.borrow().actor.get_move_ap_cost(1) {
            return false;
        }

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut().unwrap();
            let area_state = state.area_state.borrow();

            let start_time = time::Instant::now();
            let val = match state.path_finder.find(
                &area_state,
                entity.borrow(),
                entities_to_ignore,
                false,
                x,
                y,
                dist,
            ) {
                None => false,
                Some(_) => true,
            };
            trace!(
                "Path finding complete in {} secs",
                util::format_elapsed_secs(start_time.elapsed())
            );

            val
        })
    }

    pub fn party_stash() -> Rc<RefCell<PartyStash>> {
        STATE.with(|s| Rc::clone(&s.borrow().as_ref().unwrap().party_stash))
    }

    pub fn party_coins() -> i32 {
        STATE.with(|s| s.borrow().as_ref().unwrap().party_coins)
    }

    pub fn add_party_coins(amount: i32) {
        STATE.with(|s| s.borrow_mut().as_mut().unwrap().party_coins += amount);
    }

    pub fn party_formation() -> Rc<RefCell<Formation>> {
        STATE.with(|s| {
            let state = s.borrow();
            let state = state.as_ref().unwrap();

            Rc::clone(&state.party_formation)
        })
    }
}
