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
use std::io::Error;
use std::collections::HashMap;

use sulis_core::util::{gen_rand, Point};
use crate::{Module, Prop, area::{Layer, PropDataBuilder}};
use crate::generator::{GenModel, WeightedEntry, WeightedList, Maze, RegionKind, RegionKinds};

pub struct PropGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    layers: &'b [Layer],
    params: &'a PropParams,
    maze: &'b Maze,
}

impl<'a, 'b> PropGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>,
                      layers: &'b [Layer],
                      params: &'a PropParams,
                      maze: &'b Maze) -> PropGen<'a, 'b> {
        PropGen {
            model,
            layers,
            params,
            maze,
        }
    }

    pub(crate) fn generate(&mut self) -> Result<Vec<PropDataBuilder>, Error> {
        let mut props = Vec::new();

        for pass in self.params.passes.iter() {
            for _ in 0..pass.placement_attempts {
                let prop = pass.kinds.pick();
                let data = PropData::gen(self.model.builder.width as i32,
                                         self.model.builder.height as i32,
                                         prop);

                if pass.require_passable {
                    if !self.is_passable(&data) { continue; }
                }

                let (x1, y1) = self.model.to_region_coords(data.x, data.y);
                let (x2, y2) = self.model.to_region_coords(data.x + data.w(), data.y + data.h());

                let mut invalid = false;
                for y in y1..=y2 {
                    for x in x1..=x2 {
                        let t = self.maze.tile_checked(x, y);
                        if !pass.allowable_regions.is_allowable(t) {
                            invalid = true;
                            break;
                        }
                    }
                }

                if invalid { continue; }

                for other in props.iter() {
                    if data.overlaps(other, pass.spacing as i32) {
                        invalid = true;
                        break;
                    }
                }

                if invalid { continue; }

                props.push(data);
            }
        }

        let mut out = Vec::new();
        for prop in props {
            out.push(PropDataBuilder {
                id: prop.prop.id.to_string(),
                location: Point::new(prop.x, prop.y),
                items: Vec::new(),
                enabled: None,
                hover_text: None,
            });
        }
        Ok(out)
    }

    fn is_passable(&self, prop: &PropData) -> bool {
        for yi in 0..prop.h() {
            for xi in 0..prop.w() {
                let x = prop.x + xi;
                let y = prop.y + yi;
                if !self.point_is_passable(x, y) { return false; }
            }
        }

        true
    }

    fn point_is_passable(&self, x: i32, y: i32) -> bool {
        for layer in self.layers {
            if !layer.is_passable(x, y) { return false; }
        }
        true
    }
}

struct PropData {
    prop: Rc<Prop>,
    x: i32,
    y: i32,
}

impl PropData {
    fn gen(max_x: i32, max_y: i32, prop: &Rc<Prop>) -> PropData {
        let prop = Rc::clone(prop);
        let w = prop.size.width;
        let h = prop.size.height;
        let x = gen_rand(0, max_x - w);
        let y = gen_rand(0, max_y - h);

        PropData { prop, x, y }
    }

    fn w(&self) -> i32 { self.prop.size.width }

    fn h(&self) -> i32 { self.prop.size.height }

    fn overlaps(&self, other: &PropData, spacing: i32) -> bool {
        !self.not_overlaps(other, spacing)
    }

    fn not_overlaps(&self, other: &PropData, spacing: i32) -> bool {
        let sp = spacing - 1;

        if self.x > other.x + other.w() + sp || other.x > self.x + self.w() + sp {
            return true;
        }

        if self.y > other.y + other.h() + sp || other.y > self.y + self.h() + sp {
            return true;
        }

        false
    }
}

pub(crate) struct PropParams {
    passes: Vec<PropPass>,
}

impl PropParams {
    pub(crate) fn new(builder: PropParamsBuilder, module: &Module) -> Result<PropParams, Error> {
        let mut passes = Vec::new();

        for pass in builder.passes {
            let kinds = WeightedList::new(pass.kinds, "Prop",
                                          |id| module.props.get(id).map(|p| Rc::clone(p)))?;
            let regions = RegionKinds::new(pass.allowable_regions);

            passes.push(PropPass {
                kinds,
                spacing: pass.spacing,
                placement_attempts: pass.placement_attempts,
                allowable_regions: regions,
                require_passable: pass.require_passable,
            });
        }
        Ok(PropParams { passes })
    }
}

pub(crate) struct PropPass {
    kinds: WeightedList<Rc<Prop>>,
    spacing: u32,
    placement_attempts: u32,
    allowable_regions: RegionKinds,
    require_passable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PropParamsBuilder {
    passes: Vec<PropPassBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PropPassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    spacing: u32,
    placement_attempts: u32,
    allowable_regions: Vec<RegionKind>,

    #[serde(default)]
    require_passable: bool
}
