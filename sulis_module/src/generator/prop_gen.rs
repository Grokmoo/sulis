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
use crate::generator::{GenModel, WeightedEntry, WeightedList, Maze, RegionKind, RegionKinds,
    Rect, overlaps_any};

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

    pub(crate) fn generate(&mut self,
                           addn_passes: &[PropPass]) -> Result<Vec<PropDataBuilder>, Error> {
        let mut props = Vec::new();

        for pass in self.params.passes.iter().chain(addn_passes) {
            for _ in 0..pass.placement_attempts {
                let prop = pass.kinds.pick();
                let data = PropData::gen(self.model.builder.width as i32,
                                         self.model.builder.height as i32,
                                         prop);

                if pass.require_passable {
                    if !data.is_passable(&self.layers) { continue; }
                }

                let p1 = Point::from(self.model.to_region_coords(data.x, data.y));
                let p2 = Point::from(self.model.to_region_coords(data.x + data.w(),
                                                                 data.y + data.h()));

                if !pass.allowable_regions.check_coords(&self.maze, p1, p2) { continue; }

                if overlaps_any(&data, &props, pass.spacing as i32) { continue; }

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
}

struct PropData {
    prop: Rc<Prop>,
    x: i32,
    y: i32,
}

impl Rect for PropData {
    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn w(&self) -> i32 { self.prop.size.width }
    fn h(&self) -> i32 { self.prop.size.height }
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
}

pub(crate) struct PropParams {
    pub(crate) passes: Vec<PropPass>,
}

impl PropParams {
    pub(crate) fn with_module(builder: PropParamsBuilder,
                              module: &Module) -> Result<PropParams, Error> {
        PropParams::build(builder, |id| module.props.get(id).map(|p| Rc::clone(p)))
    }

    pub(crate) fn new(builder: PropParamsBuilder) -> Result<PropParams, Error> {
        PropParams::build(builder, |id| Module::prop(id))
    }

    fn build<F>(builder: PropParamsBuilder, f: F) -> Result<PropParams, Error>
        where F: Fn(&str) -> Option<Rc<Prop>> {
        let mut passes = Vec::new();

        for pass in builder.passes {
            let kinds = WeightedList::new(pass.kinds, "Prop", &f)?;
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct PropParamsBuilder {
    passes: Vec<PropPassBuilder>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PropPassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    spacing: u32,
    placement_attempts: u32,
    allowable_regions: Vec<RegionKind>,

    #[serde(default)]
    require_passable: bool
}
