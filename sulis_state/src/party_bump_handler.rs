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

use std::collections::HashSet;

use crate::{AreaState, EntityState, GameState, TurnManager};
use sulis_core::util::Point;

pub fn bump_party_overlap(area: &mut AreaState, mgr: &mut TurnManager) {
    info!("Combat initiated.  Checking for party overlap");
    let party = GameState::party();
    if party.len() < 2 {
        return;
    }

    let mut party_to_ignore = Vec::new();
    let mut bb = Vec::new();
    for member in party.iter() {
        let member = member.borrow();
        let x = member.location.x;
        let y = member.location.y;
        let w = member.size.width;
        let h = member.size.height;
        bb.push((x, y, w, h));
        party_to_ignore.push(member.index());
    }

    let mut to_bump = HashSet::new();
    for i in 0..(bb.len() - 1) {
        for j in (i + 1)..(bb.len()) {
            // if one box is on left side of the other
            if bb[i].0 >= bb[j].0 + bb[j].2 || bb[j].0 >= bb[i].0 + bb[i].2 {
                continue;
            }

            // if one box in above the other
            if bb[i].1 >= bb[j].1 + bb[j].3 || bb[j].1 >= bb[i].1 + bb[i].3 {
                continue;
            }

            debug!("Found party overlap between {} and {}", i, j);
            to_bump.insert(i);
        }
    }

    for index in to_bump {
        let member = &party[index];

        let (new, old) = {
            let member = member.borrow();
            let old = member.location.to_point();
            let new = match find_bump_position(area, &member, &party_to_ignore, old) {
                None => {
                    warn!(
                        "Unable to bump '{}' to avoid overlap",
                        member.actor.actor.name
                    );
                    continue;
                }
                Some(p) => p,
            };

            info!(
                "Bumping '{} from {:?} to {:?}",
                member.actor.actor.name, old, new
            );
            (new, old)
        };

        member.borrow_mut().location.move_to(new.x, new.y);
        area.update_entity_position(member, old.x, old.y, mgr);
        // TODO add subpos animation so move is smooth
    }
}

fn find_bump_position(
    area: &AreaState,
    entity: &EntityState,
    party: &[usize],
    cur: Point,
) -> Option<Point> {
    let to_ignore = vec![entity.index()];

    for radius in 1..4 {
        for x in -radius..radius {
            let p = Point::new(cur.x + x, cur.y - radius);
            if check_bump_position(area, entity, &to_ignore, party, p) {
                return Some(p);
            }
        }

        for y in -radius..radius {
            let p = Point::new(cur.x + radius, cur.y + y);
            if check_bump_position(area, entity, &to_ignore, party, p) {
                return Some(p);
            }
        }

        for x in -radius..=radius {
            let p = Point::new(cur.x + x, cur.y + radius);
            if check_bump_position(area, entity, &to_ignore, party, p) {
                return Some(p);
            }
        }

        for y in -radius..radius {
            let p = Point::new(cur.x - radius, cur.y + y);
            if check_bump_position(area, entity, &to_ignore, party, p) {
                return Some(p);
            }
        }
    }

    None
}

fn check_bump_position(
    area: &AreaState,
    entity: &EntityState,
    entity_vec: &[usize],
    party: &[usize],
    p: Point,
) -> bool {
    if !area.is_passable(entity, entity_vec, p.x, p.y) {
        return false;
    }

    let dest = GameState::get_point_dest(entity, p.x as f32, p.y as f32);

    let result = GameState::can_move_ignore_ap(entity, area, &party.to_vec(), dest);

    match result {
        None => false,
        Some(path) => path.len() < 10,
    }
}
