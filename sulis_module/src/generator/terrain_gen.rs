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

use sulis_core::ui::Border;
use sulis_core::util::{Point, gen_rand};
use crate::Module;
use crate::area::tile::TerrainKind;
use crate::generator::{GenModel, WeightedEntry, WeightedList, Maze, TileKind};

pub struct TerrainGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    params: &'a TerrainParams,
    maze: &'b Maze,
}

impl<'a, 'b> TerrainGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>,
                      params: &'a TerrainParams,
                      maze: &'b Maze) -> TerrainGen<'a, 'b> {
        TerrainGen {
            model,
            params,
            maze,
        }
    }

    pub fn generate(&mut self) {
        let base_terrain = self.get_terrain_tiles(self.params.base_kinds.pick());
        for p in self.model.tiles() {
            self.model.model.set_terrain_index(p.x, p.y, base_terrain);
        }

        let mut features = Vec::new();
        for i in 0..self.params.feature_passes.len() {
            self.gen_feature_pass(i, &mut features);
        }
    }

    fn gen_feature_pass(&mut self, pass: usize, features: &mut Vec<Feature>) {
        let pass = &self.params.feature_passes[pass];

        let gw = self.model.model.grid_width;
        let gh = self.model.model.grid_height;

        trace!("Performing feature pass");
        let skip = features.len();
        for _ in 0..pass.placement_attempts {
            let feature = Feature::gen(
                self.model.builder.width as i32 / gw,
                self.model.builder.height as i32 / gh,
                pass
            );

            let (x1, y1) = self.model.to_region_coords(feature.x * gw, feature.y * gh);
            let (x2, y2) = self.model.to_region_coords((feature.x + feature.w) * gw,
                                                       (feature.y + feature.h) * gh);

            let mut invalid = false;
            for y in y1..=y2 {
                for x in x1..=x2 {
                    let t = self.maze.tile_checked(x, y);
                    if !pass.is_allowable(t) {
                        invalid = true;
                        break;
                    }
                }

                if invalid { break; }
            }

            if invalid { continue; }

            for other in features.iter() {
                if feature.overlaps(other, pass) {
                    invalid = true;
                    break;
                }
            }

            if invalid { continue; }

            features.push(feature);
        }

        for feature in features.iter().skip(skip) {
            let terrain = self.get_terrain_tiles(pass.kinds.pick());

            self.do_feature_area(feature, terrain, pass.edge_underfill_chance,
                                 pass.border_walls_by);
        }
    }

    fn do_feature_area(&mut self, feature: &Feature, terrain: Option<usize>,
                       chance: u32, border_walls_by: Option<Border>) {
        let mut accum = 0;

        // north
        for x in feature.x..(feature.x + feature.w - 1) {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(x, feature.y, border_walls_by, terrain);
            }
        }

        // east
        for y in feature.y..(feature.y + feature.h - 1) {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(feature.x + feature.w - 1, y, border_walls_by, terrain);
            }
        }

        // south
        for x in ((feature.x + 1)..(feature.x + feature.w)).rev() {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(x, feature.y + feature.h - 1, border_walls_by, terrain);
            }
        }

        // west
        for y in ((feature.y + 1)..(feature.y + feature.h)).rev() {
            if self.check_terrain_chance(&mut accum, chance) {
                self.set_terrain(feature.x, y, border_walls_by, terrain);
            }
        }

        // center
        for y in (feature.y + 1)..(feature.y + feature.h) {
            for x in (feature.x + 1)..(feature.x + feature.w) {
                self.set_terrain(x, y, border_walls_by, terrain);
            }
        }
    }

    fn set_terrain(&mut self, x: i32, y: i32, border_walls_by: Option<Border>,
                   terrain: Option<usize>) {
        let model = &mut self.model.model;

        if let Some(border) = border_walls_by {
            for y_off in (-border.top)..=border.bottom {
                for x_off in (-border.right)..=border.left {
                    let xi = (x + x_off) * model.grid_width;
                    let yi = (y + y_off) * model.grid_height;

                    // model only relies on xi + yi * width > 0
                    if xi < 0 || yi < 0 { continue; }

                    if model.is_wall(xi, yi) { return; }
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
        } else {
            if gen_rand(1, 101) < chance {
                if *accum > 1 { *accum += 1; } else { *accum = 1; }
                true
            } else {
                if *accum < -1 { *accum -= 1; } else { *accum = -1; }
                false
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
    allowable_regions: [bool; 4],
    border_walls_by: Option<Border>,
}

impl FeaturePass {
    fn is_allowable(&self, kind: Option<TileKind>) -> bool {
        let index = match kind {
            Some(TileKind::Wall) => 0,
            Some(TileKind::Corridor(_)) => 1,
            Some(TileKind::Room(_)) => 2,
            Some(TileKind::DoorWay) => 3,
            None => return false,
        };

        self.allowable_regions[index]
    }
}

impl TerrainParams {
    pub(crate) fn new(builder: TerrainParamsBuilder, module: &Module) -> Result<TerrainParams, Error> {
        let base_kinds = WeightedList::new(builder.base_kinds, "TerrainKind",
                                           |id| module.terrain_kind(id))?;

        let mut passes = Vec::new();
        for pass_bldr in builder.feature_passes {
            let kinds = WeightedList::new(pass_bldr.kinds, "TerrainKind",
                                          |id| module.terrain_kind(id))?;

            let alw_src = pass_bldr.allowable_regions;
            let allowable_regions = [
                alw_src.contains(&RegionKind::Wall), alw_src.contains(&RegionKind::Corridor),
                alw_src.contains(&RegionKind::Room), alw_src.contains(&RegionKind::Doorway),
            ];

            passes.push(FeaturePass {
                kinds,
                min_size: Point::new(pass_bldr.min_size.0 as i32, pass_bldr.min_size.1 as i32),
                max_size: Point::new(pass_bldr.max_size.0 as i32, pass_bldr.max_size.1 as i32),
                spacing: pass_bldr.spacing,
                placement_attempts: pass_bldr.placement_attempts,
                edge_underfill_chance: pass_bldr.edge_underfill_chance,
                allowable_regions,
                border_walls_by: pass_bldr.border_walls_by,
            });
        }

        Ok(TerrainParams {
            base_kinds,
            feature_passes: passes
        })
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum RegionKind {
    Wall,
    Corridor,
    Room,
    Doorway,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TerrainParamsBuilder {
    base_kinds: HashMap<String, WeightedEntry>,
    feature_passes: Vec<FeaturePassBuilder>,
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
