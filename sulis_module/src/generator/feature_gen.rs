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
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use crate::generator::{
    overlaps_any, GenModel, Maze, Rect, RegionKind, RegionKinds, WeightedEntry, WeightedList,
};
use crate::{
    area::{tile::Feature, Layer},
    Module,
};
use sulis_core::util::Point;

pub struct FeatureGen<'a, 'b> {
    model: &'b mut GenModel,
    layers: &'b [Layer],
    params: &'a FeatureParams,
    maze: &'b Maze,
}

impl<'a, 'b> FeatureGen<'a, 'b> {
    pub(crate) fn new(
        model: &'b mut GenModel,
        layers: &'b [Layer],
        params: &'a FeatureParams,
        maze: &'b Maze,
    ) -> FeatureGen<'a, 'b> {
        FeatureGen {
            model,
            layers,
            params,
            maze,
        }
    }

    pub fn generate(&mut self) -> Result<(), Error> {
        let mut features = Vec::new();

        for data in &self.params.fixed {
            features.push(data.clone());
        }

        for pass in self.params.passes.iter() {
            for _ in 0..pass.placement_attempts {
                let feature = pass.kinds.pick(&mut self.model.rand);

                let (w, h) = (self.model.area_width, self.model.area_height);
                let data = FeatureData::gen(&mut self.model, w, h, feature);

                let p1 = Point::from(self.model.to_region_coords(data.x, data.y));
                let p2 = Point::from(
                    self.model
                        .to_region_coords(data.x + data.w(), data.y + data.h()),
                );

                if !pass.allowable_regions.check_coords(&self.maze, p1, p2) {
                    continue;
                }

                if overlaps_any(&data, &features, pass.spacing as i32) {
                    continue;
                }

                if pass.require_passable && !data.is_passable(&self.layers) {
                    continue;
                }

                features.push(data);
            }
        }

        for data in features {
            let feature = data.feature;
            let (base_x, base_y) = (data.x, data.y);
            for (tile, p) in feature.rand_entry() {
                self.model
                    .model
                    .add(Rc::clone(tile), base_x + p.x, base_y + p.y);
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct FeatureData {
    feature: Rc<Feature>,
    x: i32,
    y: i32,
}

impl FeatureData {
    fn gen(model: &mut GenModel, max_x: i32, max_y: i32, feature: &Rc<Feature>) -> FeatureData {
        let feature = Rc::clone(feature);
        let w = feature.size.width;
        let h = feature.size.height;
        let x = model.rand.gen(0, max_x - w);
        let y = model.rand.gen(0, max_y - h);

        FeatureData { feature, x, y }
    }
}

impl Rect for FeatureData {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn w(&self) -> i32 {
        self.feature.size.width
    }
    fn h(&self) -> i32 {
        self.feature.size.height
    }
}

pub(crate) struct FeatureParams {
    passes: Vec<FeaturePass>,
    fixed: Vec<FeatureData>,
}

impl FeatureParams {
    pub(crate) fn new(
        builder: FeatureParamsBuilder,
        module: &Module,
    ) -> Result<FeatureParams, Error> {
        let mut passes = Vec::new();

        for pass in builder.passes {
            let kinds = WeightedList::new(pass.kinds, "Feature", |id| {
                module.features.get(id).map(|f| Rc::clone(f))
            })?;
            let regions = RegionKinds::new(pass.allowable_regions);

            passes.push(FeaturePass {
                kinds,
                spacing: pass.spacing,
                placement_attempts: pass.placement_attempts,
                allowable_regions: regions,
                require_passable: pass.require_passable,
            })
        }

        let mut fixed = Vec::new();
        for (id, p) in builder.fixed {
            let feature = module.features.get(&id).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid feature '{}' in gen fixed.", id),
                )
            })?;
            fixed.push(FeatureData {
                feature: Rc::clone(feature),
                x: p.x,
                y: p.y,
            });
        }

        Ok(FeatureParams { passes, fixed })
    }
}

pub(crate) struct FeaturePass {
    kinds: WeightedList<Rc<Feature>>,
    spacing: u32,
    placement_attempts: u32,
    allowable_regions: RegionKinds,
    require_passable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeatureParamsBuilder {
    passes: Vec<FeaturePassBuilder>,
    fixed: Vec<(String, Point)>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeaturePassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    spacing: u32,
    placement_attempts: u32,
    allowable_regions: Vec<RegionKind>,
    require_passable: bool,
}
