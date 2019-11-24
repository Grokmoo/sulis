//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use sulis_core::util::Point;
use sulis_module::{Area, area::{ToKind, TriggerKind}, ObjectSize, Time};
use crate::{AreaState, EntityState, GameState, Location, TurnManager};

pub(crate) fn transition_to(area_id: Option<&str>, p: Option<Point>, offset: Point, time: Time) {
    info!("Area transition to {:?}: {:?}", area_id, p);

    if let Some(id) = area_id {
        if let Err(e) = GameState::preload_area(id) {
            error!("Error loading {} while transitioning", id);
            error!("{}", e);
            return;
        }
    }

    let (area, location) = match get_area(area_id, p) {
        None => return,
        Some((area, location)) => (area, location),
    };

    let p = Point::new(location.x + offset.x, location.y + offset.y);

    {
        let area = &area.borrow().area.area;
        if !check_location(p, area) {
            error!("Invalid transition location {:?} in {}", location, area.id);
            return;
        }
    }

    // Point of no return - we are actually transitioning now

    GameState::set_current_area(&area);
    GameState::set_clear_anims(); // cleanup anims and surfaces

    let mgr = GameState::turn_manager();
    let party = GameState::party();
    let area = GameState::area_state(); // it changed above in set_current_area

    remove_party_from_surfaces(&mut mgr.borrow_mut(), &party);

    remove_party_auras(&mut mgr.borrow_mut(), &party);

    mgr.borrow_mut().add_time(time);

    transition_party(&mgr, &area, p, &party);

    let pc = GameState::player();
    area.borrow_mut().push_scroll_to_callback(Rc::clone(&pc));

    let mut area = area.borrow_mut();
    area.update_view_visibility();
    if !area.on_load_fired {
        area.on_load_fired = true;
        GameState::add_ui_callbacks_of_kind(
            &area.area.area.triggers,
            TriggerKind::OnAreaLoad,
            &pc,
            &pc
        );
    }
}

fn transition_party(
    mgr: &Rc<RefCell<TurnManager>>,
    area: &Rc<RefCell<AreaState>>,
    p: Point,
    party: &[Rc<RefCell<EntityState>>]
) {
    let base_location = Location::new(p.x, p.y, &area.borrow().area.area);

    for entity in party {
        entity.borrow_mut().clear_pc_vis();
        let mut cur_location = base_location.clone();
        find_transition_location(&mut cur_location, &entity.borrow().size, &area.borrow());

        info!("Transitioning '{}' to {:?}", entity.borrow().actor.actor.name, cur_location);

        let (index, dx, dy) = {
            let entity = entity.borrow();
            (entity.index(), entity.location.x - cur_location.x, entity.location.y - cur_location.y)
        };

        add_member_auras(&mut mgr.borrow_mut(), &mut area.borrow_mut(), index, dx, dy);

        if let Err(e) = area.borrow_mut().transition_entity_to(entity, index, cur_location) {
            warn!("Unable to add party member '{}'", entity.borrow().actor.actor.id);
            warn!("{}", e);
        }
    }
}

pub fn find_transition_location(location: &mut Location, size: &ObjectSize, area: &AreaState) {
    let (base_x, base_y) = (location.x, location.y);
    let mut search_size = 0;
    while search_size < 10 {
        // TODO a lot of duplicate effort here
        for y in -search_size..search_size + 1 {
            for x in -search_size..search_size + 1 {
                if area.is_passable_size(size, base_x + x, base_y + y) {
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

fn add_member_auras(mgr: &mut TurnManager, area: &mut AreaState, index: usize, dx: i32, dy: i32) {
    let aura_indices = mgr.auras_for(index);
    for aura_index in aura_indices {
        let aura = mgr.effect_mut(aura_index);
        let surface = match aura.surface {
            None => continue,
            Some(ref mut surface) => surface,
        };
        surface.area_id = area.area.area.id.to_string();
        for ref mut p in surface.points.iter_mut() {
            p.x -= dx;
            p.y -= dy;
        }

        let to_add = area.add_surface(aura_index, &surface.points);
        for entity in to_add {
            mgr.add_to_surface(entity, aura_index);
        }
    }
}

fn remove_party_auras(mgr: &mut TurnManager, party: &[Rc<RefCell<EntityState>>]) {
    for entity in party {
        let area = entity_area(&entity.borrow());
        let mut area = area.borrow_mut();
        let entity_index = entity.borrow().index();
        let aura_indices = mgr.auras_for(entity_index);
        for aura_index in aura_indices {
            let aura = mgr.effect_mut(aura_index);
            let surface = match aura.surface {
                None => continue,
                Some(ref mut surface) => surface,
            };

            let to_remove = area.remove_surface(aura_index, &surface.points);
            for entity in to_remove {
                mgr.remove_from_surface(entity, aura_index);
            }
        }
    }
}

fn remove_party_from_surfaces(mgr: &mut TurnManager, party: &[Rc<RefCell<EntityState>>]) {
 for entity in party {
     let area = entity_area(&entity.borrow());

     let surfaces = area.borrow_mut().remove_entity(entity, mgr);
     let entity_index = entity.borrow().index();
     for surface in surfaces {
         mgr.remove_from_surface(entity_index, surface);
     }
 }
}

fn entity_area(entity: &EntityState) -> Rc<RefCell<AreaState>> {
    let id = &entity.location.area_id;
    GameState::get_area_state(id).unwrap()
}

fn get_area(area_id: Option<&str>, p: Option<Point>) -> Option<(Rc<RefCell<AreaState>>, Point)> {
    match area_id {
        None => {
            let area = GameState::area_state();
            match p {
                None => {
                    error!("No point specified for intra area transition");
                    return None;
                }, Some(p) => {
                    Some((area, p))
                }
            }
        }, Some(id) => {
            let state = match GameState::get_area_state(id) {
                None => {
                    error!("Invalid area id '{}' in transition", id);
                    return None;
                }, Some(state) => state,
            };

            let location = match p {
                Some(p) => p,
                None => {
                    let old = GameState::area_state();
                    let old = old.borrow();
                    match find_link(&state.borrow(), &old.area.area.id) {
                        None => {
                            error!("Error finding linked coords for transition {}", id);
                            return None;
                        },
                        Some(location) => location,
                    }
                }
            };
            Some((state, location))
        }
    }
}

fn find_link(state: &AreaState, from: &str) -> Option<Point> {
    for transition in state.area.transitions.iter() {
        match transition.to {
            ToKind::Area { ref id, .. } | ToKind::FindLink { ref id, .. } => {
                if id == from {
                    return Some(transition.from);
                }
            }
            _ => (),
        }
    }

    None
}

fn check_location(p: Point, area: &Area) -> bool {
    let location = Location::from_point(&p, area);
    if !location.coords_valid(location.x, location.y) {
        error!(
            "Location coordinates {},{} are not valid for area {}",
            location.x, location.y, location.area_id
        );
        return false;
    }

    true
}
