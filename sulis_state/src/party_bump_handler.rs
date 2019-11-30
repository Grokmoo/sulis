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

use sulis_core::util::Point;
use crate::{AreaState, EntityState, GameState, TurnManager};

pub fn bump_party_overlap(area: &mut AreaState, mgr: &mut TurnManager) {
    info!("Combat initiated.  Checking for party overlap");
    let party = GameState::party();
    if party.len() < 2 {
        return;
    }

    let mut bb = Vec::new();
    for member in party.iter() {
        let member = member.borrow();
        let x = member.location.x;
        let y = member.location.y;
        let w = member.size.width;
        let h = member.size.height;
        bb.push((x, y, w, h));
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
            let new = match find_bump_position(area, &member, old) {
                None => {
                    warn!("Unable to bump '{}' to avoid overlap", member.actor.actor.name);
                    continue;
                }, Some(p) => p,
            };

            info!("Bumping '{} from {:?} to {:?}", member.actor.actor.name, old, new);
            (new, old)
        };

        member.borrow_mut().location.move_to(new.x, new.y);
        area.update_entity_position(member, old.x, old.y, mgr);
        // TODO add subpos animation so move is smooth
    }
}

fn find_bump_position(area: &AreaState, entity: &EntityState, cur: Point) -> Option<Point> {
    let to_ignore = vec![entity.index()];
    for radius in 1..=3 {
        for y in -radius..=radius {
            for x in -radius..=radius {
                let p = Point::new(cur.x + x, cur.y + y);
                if area.is_passable(entity, &to_ignore, p.x, p.y) {
                    return Some(p);
                }
            }
        }
    }
    None
}
