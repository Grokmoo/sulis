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

use std::io::Error;
use std::rc::Rc;

use crate::{
    area::tile::{Tile, WallKind, WallRules},
    generator::EdgesList,
    Module,
};

#[derive(Clone)]
pub struct WallTiles {
    pub id: String,
    pub fill_tile: Option<Rc<Tile>>,

    pub edges: EdgesList,
    pub extended: Vec<EdgesList>,
    pub interior_border: bool,
}

impl WallTiles {
    pub fn new(kind: WallKind, rules: &WallRules) -> Result<WallTiles, Error> {
        let fill_tile = match kind.fill_tile {
            None => None,
            Some(ref fill_tile) => {
                let fill_tile_id = format!("{}{}{}", &rules.prefix, &kind.id, fill_tile);
                match Module::tile(&fill_tile_id) {
                    None => {
                        warn!("No fill tile found for '{}'", kind.id);
                        None
                    }
                    Some(tile) => Some(tile),
                }
            }
        };

        let edges = EdgesList::new(&kind.id, &rules.prefix, &rules.edges)?;

        let mut extended = Vec::new();
        for prefix in kind.extended {
            let e = EdgesList::new(&prefix, &rules.prefix, &rules.edges)?;
            extended.push(e);
        }

        Ok(WallTiles {
            id: kind.id,
            edges,
            extended,
            fill_tile,
            interior_border: kind.interior_border,
        })
    }
}
