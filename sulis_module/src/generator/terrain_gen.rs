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
use std::io::{Error};

use sulis_core::util::{Point, gen_rand};
use crate::Module;
use crate::area::tile::TerrainKind;
use crate::generator::{GenModel, WeightedEntry, WeightedList};

pub struct TerrainGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    params: &'a TerrainParams,
}

impl<'a, 'b> TerrainGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>, params: &'a TerrainParams) -> TerrainGen<'a, 'b> {
        TerrainGen {
            model,
            params,
        }
    }

    pub fn generate(&mut self) {
        let base_terrain = self.get_terrain_tiles(self.params.base_kinds.pick());
        for p in self.model.tiles() {
            self.model.model.set_terrain_index(p.x, p.y, base_terrain);
        }

        for i in 0..self.params.feature_passes.len() {
            self.gen_feature_pass(i);
        }
    }

    fn gen_feature_pass(&mut self, pass: usize) {
        let pass = &self.params.feature_passes[pass];

        let mut features = Vec::new();
        for _ in 0..pass.placement_attempts {
            let feature = Feature::gen(self.model.builder.width as i32 / self.model.model.grid_width,
                                       self.model.builder.height as i32 / self.model.model.grid_height,
                                       pass);

            let mut overlaps = false;
            for other in &features {
                if feature.overlaps(other, pass) {
                    overlaps = true;
                    break;
                }
            }

            if overlaps { continue; }

            features.push(feature);
        }

        for feature in features {
            let terrain = self.get_terrain_tiles(pass.kinds.pick());

            self.do_feature_area(feature, terrain, pass.edge_underfill_chance);
        }
    }

    fn do_feature_area(&mut self, feature: Feature, terrain: Option<usize>, chance: u32) {
        let mut accum = 0;

        // north
        for x in feature.x..(feature.x + feature.w - 1) {
            self.set_terrain_chance(&mut accum, chance, x, feature.y, terrain);
        }

        // east
        for y in feature.y..(feature.y + feature.h - 1) {
            self.set_terrain_chance(&mut accum, chance, feature.x + feature.w - 1, y, terrain);
        }

        // south
        for x in ((feature.x + 1)..(feature.x + feature.w)).rev() {
            self.set_terrain_chance(&mut accum, chance, x, feature.y + feature.h - 1, terrain);
        }

        // west
        for y in ((feature.y + 1)..(feature.y + feature.h)).rev() {
            self.set_terrain_chance(&mut accum, chance, feature.x, y, terrain);
        }

        // center
        for y in (feature.y + 1)..(feature.y + feature.h - 1) {
            for x in (feature.x + 1)..(feature.x + feature.w - 1) {
                let x = x * self.model.model.grid_width;
                let y = y * self.model.model.grid_height;

                self.model.model.set_terrain_index(x, y, terrain);
            }
        }
    }

    fn set_terrain_chance(&mut self, accum: &mut i32, chance: u32,
                          x: i32, y: i32, terrain: Option<usize>) {
        let x = x * self.model.model.grid_width;
        let y = y * self.model.model.grid_height;

        if *accum == 1 || *accum == 2 {
            self.model.model.set_terrain_index(x, y, terrain);
            *accum += 1;
        } else if *accum == -1 || *accum == -2 {
            *accum -= 1;
        } else {
            if gen_rand(1, 101) < chance {
                self.model.model.set_terrain_index(x, y, terrain);
                if *accum > 1 { *accum += 1; } else { *accum = 1; }
            } else {
                if *accum < -1 { *accum -= 1; } else { *accum = -1; }
            }
        }
    }

    fn get_terrain_tiles(&self, kind: &TerrainKind) -> Option<usize> {
        let model = &self.model.model;

        for (index, possible_kind) in model.terrain_kinds().iter().enumerate() {
            if kind.id == possible_kind.id {
                return Some(index);
            }
        }

        error!("Invalid terrain kind '{}'.  This is a bug.", kind.id);
        panic!()
    }
}

pub(crate) struct TerrainParams {
    base_kinds: WeightedList<TerrainKind>,
    feature_passes: Vec<FeaturePass>,
}

pub(crate) struct FeaturePass {
    kinds: WeightedList<TerrainKind>,
    min_size: Point,
    max_size: Point,
    spacing: u32,
    placement_attempts: u32,
    edge_underfill_chance: u32,
}

impl TerrainParams {
    pub(crate) fn new(builder: TerrainParamsBuilder, module: &Module) -> Result<TerrainParams, Error> {
        let base_kinds = WeightedList::new(builder.base_kinds, "TerrainKind",
                                           |id| module.terrain_kind(id))?;

        let mut passes = Vec::new();
        for (_, pass_bldr) in builder.feature_passes {
            let kinds = WeightedList::new(pass_bldr.kinds, "TerrainKind",
                                          |id| module.terrain_kind(id))?;
            passes.push(FeaturePass {
                kinds,
                min_size: Point::new(pass_bldr.min_size.0 as i32, pass_bldr.min_size.1 as i32),
                max_size: Point::new(pass_bldr.max_size.0 as i32, pass_bldr.max_size.1 as i32),
                spacing: pass_bldr.spacing,
                placement_attempts: pass_bldr.placement_attempts,
                edge_underfill_chance: pass_bldr.edge_underfill_chance,
            });
        }


        Ok(TerrainParams {
            base_kinds,
            feature_passes: passes
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TerrainParamsBuilder {
    base_kinds: HashMap<String, WeightedEntry>,
    feature_passes: HashMap<String, FeaturePassBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeaturePassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    min_size: (u32, u32),
    max_size: (u32, u32),
    spacing: u32,
    placement_attempts: u32,
    edge_underfill_chance: u32,
}

struct Feature {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Feature {
    fn gen(max_x: i32, max_y: i32, params: &FeaturePass) -> Feature {
        let w = gen_rand(params.min_size.x, params.max_size.x + 1);
        let h = gen_rand(params.min_size.y, params.max_size.y + 1);
        let x = gen_rand(0, max_x - w);
        let y = gen_rand(0, max_y - h);

        Feature { x, y, w, h }
    }

    fn overlaps(&self, other: &Feature, params: &FeaturePass) -> bool {
        !self.not_overlaps(other, params)
    }

    fn not_overlaps(&self, other: &Feature, params: &FeaturePass) -> bool {
        let sp = params.spacing as i32 - 1;

        if self.x > other.x + other.w + sp || other.x > self.x + self.w + sp {
            return true;
        }

        if self.y > other.y + other.h + sp || other.y > self.y + self.h + sp {
            return true;
        }

        false
    }
}
