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

use serde::Deserialize;

use crate::area::tile::TerrainKind;
use crate::generator::{
    overlaps_any, GenModel, Maze, Rect, RegionKind, RegionKinds, WeightedEntry, WeightedList,
};
use crate::Module;
use sulis_core::ui::Border;
use sulis_core::util::Point;

pub struct TerrainGen<'a, 'b> {
    model: &'b mut GenModel,
    params: &'a TerrainParams,
    maze: &'b Maze,
}

impl<'a, 'b> TerrainGen<'a, 'b> {
    pub(crate) fn new(
        model: &'b mut GenModel,
        params: &'a TerrainParams,
        maze: &'b Maze,
    ) -> TerrainGen<'a, 'b> {
        TerrainGen {
            model,
            params,
            maze,
        }
    }

    pub fn generate(&mut self) {
        let picks = self.params.base_kinds.pick(&mut self.model.rand);
        let base_terrain = self.get_terrain_tiles(picks);
        for p in self.model.tiles() {
            self.model.model.set_terrain_index(p.x, p.y, base_terrain);
        }

        let mut patches = Vec::new();
        for i in 0..self.params.patch_passes.len() {
            self.gen_patch_pass(i, &mut patches);
        }
    }

    fn gen_patch_pass(&mut self, pass: usize, patches: &mut Vec<Feature>) {
        let pass = &self.params.patch_passes[pass];

        let gw = self.model.model.grid_width;
        let gh = self.model.model.grid_height;

        trace!("Performing patch pass");
        let skip = patches.len();
        for _ in 0..pass.placement_attempts {
            let (w, h) = (self.model.area_width, self.model.area_height);
            let patch = Feature::gen(self.model, w, h, pass);

            let (x1, y1) = self.model.to_region_coords(patch.x * gw, patch.y * gh);
            let (x2, y2) = self
                .model
                .to_region_coords((patch.x + patch.w) * gw, (patch.y + patch.h) * gh);
            let p1 = Point::from((x1, y1));
            let p2 = Point::from((x2, y2));

            if !pass.allowable_regions.check_coords(self.maze, p1, p2) {
                continue;
            }

            if overlaps_any(&patch, patches, pass.spacing as i32) {
                continue;
            }

            patches.push(patch);
        }

        for patch in patches.iter().skip(skip) {
            let picks = pass.kinds.pick(&mut self.model.rand);
            let terrain = self.get_terrain_tiles(picks);

            self.do_patch_area(
                patch,
                terrain,
                pass.edge_underfill_chance,
                pass.border_walls_by,
            );
        }
    }

    fn do_patch_area(
        &mut self,
        patch: &Feature,
        terrain: Option<usize>,
        chance: u32,
        border_walls_by: Option<Border>,
    ) {
        let mut accum = 0;

        // north
        for x in patch.x..(patch.x + patch.w - 1) {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(x, patch.y, border_walls_by, terrain);
            }
        }

        // east
        for y in patch.y..(patch.y + patch.h - 1) {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(patch.x + patch.w - 1, y, border_walls_by, terrain);
            }
        }

        // south
        for x in ((patch.x + 1)..(patch.x + patch.w)).rev() {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(x, patch.y + patch.h - 1, border_walls_by, terrain);
            }
        }

        // west
        for y in ((patch.y + 1)..(patch.y + patch.h)).rev() {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(patch.x, y, border_walls_by, terrain);
            }
        }

        // center
        for y in (patch.y + 1)..(patch.y + patch.h) {
            for x in (patch.x + 1)..(patch.x + patch.w) {
                self.set_terrain(x, y, border_walls_by, terrain);
            }
        }
    }

    fn set_terrain(
        &mut self,
        x: i32,
        y: i32,
        border_walls_by: Option<Border>,
        terrain: Option<usize>,
    ) {
        let model = &mut self.model.model;

        if let Some(border) = border_walls_by {
            for y_off in (-border.top)..=border.bottom {
                for x_off in (-border.right)..=border.left {
                    let xi = (x + x_off) * model.grid_width;
                    let yi = (y + y_off) * model.grid_height;

                    // model only relies on xi + yi * width > 0
                    if xi < 0 || yi < 0 {
                        continue;
                    }

                    if model.is_wall(xi, yi) {
                        return;
                    }
                }
            }
        }

        let x = x * model.grid_width;
        let y = y * model.grid_height;
        model.set_terrain_index(x, y, terrain);
    }

    fn check_terrain_chance(&mut self, accum: &mut i32, chance: u32) -> bool {
        if *accum == 1 || *accum == 2 {
            *accum += 1;
            true
        } else if *accum == -1 || *accum == -2 {
            *accum -= 1;
            false
        } else if self.model.rand.gen(1, 101) < chance {
            if *accum > 1 {
                *accum += 1;
            } else {
                *accum = 1;
            }
            true
        } else {
            if *accum < -1 {
                *accum -= 1;
            } else {
                *accum = -1;
            }
            false
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
    patch_passes: Vec<FeaturePass>,
}

pub(crate) struct FeaturePass {
    kinds: WeightedList<TerrainKind>,
    min_size: Point,
    max_size: Point,
    spacing: u32,
    placement_attempts: u32,
    edge_underfill_chance: u32,
    allowable_regions: RegionKinds,
    border_walls_by: Option<Border>,
}

impl TerrainParams {
    pub(crate) fn new(
        builder: TerrainParamsBuilder,
        module: &Module,
    ) -> Result<TerrainParams, Error> {
        let base_kinds = WeightedList::new(builder.base_kinds, "TerrainKind", |id| {
            module.terrain_kind(id)
        })?;

        let mut passes = Vec::new();
        for pass_bldr in builder.patch_passes {
            let kinds =
                WeightedList::new(pass_bldr.kinds, "TerrainKind", |id| module.terrain_kind(id))?;

            passes.push(FeaturePass {
                kinds,
                min_size: Point::new(pass_bldr.min_size.0 as i32, pass_bldr.min_size.1 as i32),
                max_size: Point::new(pass_bldr.max_size.0 as i32, pass_bldr.max_size.1 as i32),
                spacing: pass_bldr.spacing,
                placement_attempts: pass_bldr.placement_attempts,
                edge_underfill_chance: pass_bldr.edge_underfill_chance,
                allowable_regions: RegionKinds::new(pass_bldr.allowable_regions),
                border_walls_by: pass_bldr.border_walls_by,
            });
        }

        Ok(TerrainParams {
            base_kinds,
            patch_passes: passes,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TerrainParamsBuilder {
    base_kinds: HashMap<String, WeightedEntry>,
    patch_passes: Vec<FeaturePassBuilder>,
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
    allowable_regions: Vec<RegionKind>,
    border_walls_by: Option<Border>,
}

struct Feature {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect for Feature {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn w(&self) -> i32 {
        self.w
    }
    fn h(&self) -> i32 {
        self.h
    }
}

impl Feature {
    fn gen(model: &mut GenModel, max_x: i32, max_y: i32, params: &FeaturePass) -> Feature {
        let w = model.rand.gen(params.min_size.x, params.max_size.x + 1);
        let h = model.rand.gen(params.min_size.y, params.max_size.y + 1);
        let x = model.rand.gen(0, max_x - w);
        let y = model.rand.gen(0, max_y - h);

        Feature { x, y, w, h }
    }
}
