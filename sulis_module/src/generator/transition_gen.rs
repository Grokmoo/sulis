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
use std::io::{ErrorKind, Error};
use std::rc::Rc;

use sulis_core::util::{gen_rand};
use crate::{Module, area::{Layer, tile::{Feature}}};
use crate::generator::{GenModel, Maze, RegionKind,
    RegionKinds, Rect};

pub struct TransitionsGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    layers: &'b [Layer],
    params: &'a TransitionsParams,
    maze: &'b Maze,
}

impl<'a, 'b> TransitionsGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>,
                      layers: &'b [Layer],
                      params: &'a TransitionsParams,
                      maze: &'b Maze) -> TransitionsGen<'a, 'b> {
        TransitionsGen {
            model,
            layers,
            params,
            maze,
        }
    }

    pub fn generate(&mut self) -> Result<(), Error> {

        Ok(())
    }
}

#[derive(Clone)]
struct TransitionData {
    feature: Rc<Feature>,
    x: i32,
    y: i32,
}

impl TransitionData {
    fn gen(max_x: i32, max_y: i32, feature: &Rc<Feature>) -> TransitionData {
        let feature = Rc::clone(feature);
        let w = feature.size.width;
        let h = feature.size.height;
        let x = gen_rand(0, max_x - w);
        let y= gen_rand(0, max_y - h);

        TransitionData { feature, x, y }
    }
}

impl Rect for TransitionData {
    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn w(&self) -> i32 { self.feature.size.width }
    fn h(&self) -> i32 { self.feature.size.height }
}

pub(crate) struct TransitionsParams {
    kinds: HashMap<String, TransitionsKind>,
}

impl TransitionsParams {
    pub(crate) fn new(builder: TransitionsParamsBuilder,
                      module: &Module) -> Result<TransitionsParams, Error> {
        let mut kinds = HashMap::new();

        for (id, kind) in builder.kinds {
            let regions = RegionKinds::new(kind.allowable_regions);

            let feature = match kind.feature {
                None => None,
                Some(feature_id) => {
                    Some(Rc::clone(module.features.get(&id).ok_or(
                        Error::new(ErrorKind::InvalidInput, format!("Invalid feature '{}'",
                                                                    feature_id))
                    )?))
                }
            };

            kinds.insert(id, TransitionsKind {
                allowable_regions: regions,
                feature,
            });
        }

        Ok(TransitionsParams { kinds })
    }
}

pub(crate) struct TransitionsKind {
    allowable_regions: RegionKinds,
    feature: Option<Rc<Feature>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionsParamsBuilder {
    kinds: HashMap<String, TransitionsKindBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionsKindBuilder {
    allowable_regions: Vec<RegionKind>,
    feature: Option<String>
}
