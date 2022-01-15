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

use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use crate::{
    area::tile::{EdgeRules, TerrainKind, TerrainRules, Tile},
    Module,
};
use sulis_core::util::unable_to_create_error;

#[derive(Clone)]
pub struct TerrainTiles {
    pub id: String,
    pub base: Rc<Tile>,
    pub base_weight: u32,
    pub variants: Vec<Rc<Tile>>,
    pub edges: EdgesList,
    pub borders: HashMap<usize, EdgesList>,
}

impl PartialEq for TerrainTiles {
    fn eq(&self, other: &TerrainTiles) -> bool {
        self.id == other.id
    }
}

impl Eq for TerrainTiles {}

#[derive(Clone)]
pub struct EdgesList {
    pub inner_nw: Option<Rc<Tile>>,
    pub inner_ne: Option<Rc<Tile>>,
    pub inner_sw: Option<Rc<Tile>>,
    pub inner_se: Option<Rc<Tile>>,

    pub outer_n: Option<Rc<Tile>>,
    pub outer_s: Option<Rc<Tile>>,
    pub outer_e: Option<Rc<Tile>>,
    pub outer_w: Option<Rc<Tile>>,
    pub outer_se: Option<Rc<Tile>>,
    pub outer_ne: Option<Rc<Tile>>,
    pub outer_sw: Option<Rc<Tile>>,
    pub outer_nw: Option<Rc<Tile>>,

    pub outer_all: Option<Rc<Tile>>,
    pub inner_ne_sw: Option<Rc<Tile>>,
    pub inner_nw_se: Option<Rc<Tile>>,
}

impl EdgesList {
    pub fn new(id: &str, prefix: &str, rules: &EdgeRules) -> Result<EdgesList, Error> {
        let inner_nw =
            EdgesList::get_edge(prefix, id, &rules.inner_edge_postfix, &rules.nw_postfix);
        let inner_ne =
            EdgesList::get_edge(prefix, id, &rules.inner_edge_postfix, &rules.ne_postfix);
        let inner_sw =
            EdgesList::get_edge(prefix, id, &rules.inner_edge_postfix, &rules.sw_postfix);
        let inner_se =
            EdgesList::get_edge(prefix, id, &rules.inner_edge_postfix, &rules.se_postfix);

        let outer_n =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.n_postfix);
        let outer_s =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.s_postfix);
        let outer_e =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.e_postfix);
        let outer_w =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.w_postfix);
        let outer_ne =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.ne_postfix);
        let outer_nw =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.nw_postfix);
        let outer_se =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.se_postfix);
        let outer_sw =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.sw_postfix);
        let outer_all =
            EdgesList::get_edge(prefix, id, &rules.outer_edge_postfix, &rules.all_postfix);

        let inner_ne_sw = EdgesList::get_edge(
            prefix,
            id,
            &rules.inner_edge_postfix,
            &rules.ne_sw_postfix,
        );

        let inner_nw_se = EdgesList::get_edge(
            prefix,
            id,
            &rules.inner_edge_postfix,
            &rules.nw_se_postfix,
        );

        Ok(EdgesList {
            inner_nw,
            inner_ne,
            inner_sw,
            inner_se,
            outer_n,
            outer_s,
            outer_e,
            outer_w,
            outer_se,
            outer_ne,
            outer_sw,
            outer_nw,
            outer_all,
            inner_ne_sw,
            inner_nw_se,
        })
    }

    fn get_edge(prefix: &str, id: &str, edge_postfix: &str, dir_postfix: &str) -> Option<Rc<Tile>> {
        let tile_id = format!("{}{}{}{}", prefix, id, edge_postfix, dir_postfix);

        match Module::tile(&tile_id) {
            None => {
                trace!(
                    "Edge tile with '{}', '{}' not found for '{}'.  Full path: '{}'",
                    edge_postfix,
                    dir_postfix,
                    id,
                    tile_id,
                );
                None
            }
            Some(tile) => Some(tile),
        }
    }
}

impl TerrainTiles {
    pub fn new(
        rules: &TerrainRules,
        kind: &TerrainKind,
        all_kinds: &[TerrainKind],
    ) -> Result<TerrainTiles, Error> {
        let base_tile_id = format!("{}{}{}", rules.prefix, kind.id, rules.base_postfix);
        let base = match Module::tile(&base_tile_id) {
            None => {
                warn!("Base tile for terrain kind '{}' not found", kind.id);
                return unable_to_create_error("terrain_tiles", &kind.id);
            }
            Some(tile) => tile,
        };

        let base_weight = match kind.base_weight {
            None => rules.base_weight,
            Some(weight) => weight,
        };

        let mut variants = Vec::new();
        for i in kind.variants.iter() {
            let tile_id = format!(
                "{}{}{}{}",
                rules.prefix,
                kind.id,
                rules.variant_postfix,
                i
            );
            let tile = match Module::tile(&tile_id) {
                None => {
                    warn!(
                        "Tile variant '{}' not found for terrain kind '{}'",
                        i, kind.id
                    );
                    continue;
                }
                Some(tile) => tile,
            };
            variants.push(tile);
        }

        let mut borders = HashMap::new();
        for (other_terrain, id) in kind.borders.iter() {
            let edges = EdgesList::new(id, &rules.prefix, &rules.edges)?;

            let mut index = None;
            for (i, other_kind) in all_kinds.iter().enumerate() {
                if &other_kind.id == other_terrain {
                    index = Some(i);
                    break;
                }
            }

            match index {
                None => {
                    warn!(
                        "Other terrain '{}' not found for border of '{}'",
                        other_terrain, kind.id
                    );
                    continue;
                }
                Some(index) => {
                    borders.insert(index, edges);
                }
            }
        }

        let edges = EdgesList::new(&kind.id, &rules.prefix, &rules.edges)?;

        Ok(TerrainTiles {
            id: kind.id.clone(),
            base,
            base_weight,
            variants,
            borders,
            edges,
        })
    }

    pub fn matching_edges(&self, index: Option<usize>) -> &EdgesList {
        match index {
            None => &self.edges,
            Some(index) => match self.borders.get(&index) {
                None => &self.edges,
                Some(edges) => edges,
            },
        }
    }
}
