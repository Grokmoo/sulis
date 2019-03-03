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

use sulis_core::util::{gen_rand, Point};
use crate::{Module, ObjectSize,
    area::{TransitionBuilder, ToKind, TransitionAreaParams, tile::{Feature}}};
use crate::generator::{GenModel, Rect, overlaps_any};

pub struct TransitionGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    params: &'a TransitionParams,
}

impl<'a, 'b> TransitionGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>,
                      params: &'a TransitionParams) -> TransitionGen<'a, 'b> {
        TransitionGen {
            model,
            params,
        }
    }

    pub fn generate(&mut self, transitions: &[TransitionAreaParams])
        -> Result<Vec<TransitionBuilder>, Error> {

        let mut gened = Vec::new();
        let mut out = Vec::new();
        for transition in transitions {
            let kind = self.params.kinds.get(&transition.kind).ok_or(
                Error::new(ErrorKind::InvalidInput, format!("Invalid transition kind '{}'",
                                                            transition.kind))
            )?;

            // keep trying until we succeed.  we always want to place transitions
            let mut placed = false;
            for _ in 0..1000 {
                let data = TransitionData::gen(self.model.builder.width as i32,
                                               self.model.builder.height as i32,
                                               kind.size.width,
                                               kind.size.height,
                                               &transition.to,
                                               &kind.feature);

                if overlaps_any(&data, &gened, self.params.spacing as i32) { continue; }

                if let Some(feature) = &kind.feature {
                    let base_x = data.x + kind.feature_offset.x;
                    let base_y = data.y + kind.feature_offset.y;
                    for (tile, p) in feature.rand_entry() {
                        self.model.model.add(Rc::clone(tile), base_x + p.x, base_y + p.y);
                    }
                }

                out.push(TransitionBuilder {
                    from: Point::new(data.x, data.y),
                    size: kind.size.id.to_string(),
                    to: ToKind::Area {
                        id: transition.to.to_string(),
                        x: 0,
                        y: 0,
                    },
                    hover_text: transition.hover_text.to_string(),
                    image_display: "empty".to_string(),
                });

                placed = true;
                gened.push(data);
                break;
            }

            if !placed {
                return Err(Error::new(ErrorKind::InvalidData, "Unable to place transition"));
            }
        }

        Ok(out)
    }
}

#[derive(Clone)]
struct TransitionData<'a> {
    feature: Option<Rc<Feature>>,
    to: &'a str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl<'a> TransitionData<'a> {
    fn gen(max_x: i32, max_y: i32, w: i32, h: i32,
           to: &'a str,
           feature: &Option<Rc<Feature>>) -> TransitionData<'a> {
        let feature = feature.clone();
        let x = gen_rand(0, max_x - w);
        let y= gen_rand(0, max_y - h);

        TransitionData { feature, to, x, y, w, h }
    }
}

impl<'a> Rect for TransitionData<'a> {
    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn w(&self) -> i32 { self.w }
    fn h(&self) -> i32 { self.h }
}

pub(crate) struct TransitionParams {
    spacing: u32,
    kinds: HashMap<String, TransitionKind>,
}

impl TransitionParams {
    pub(crate) fn new(builder: TransitionParamsBuilder,
                      module: &Module) -> Result<TransitionParams, Error> {
        let mut kinds = HashMap::new();

        for (id, kind) in builder.kinds {
            let feature = match kind.feature {
                None => None,
                Some(feature_id) => {
                    Some(module.features.get(&feature_id).map(|f| Rc::clone(f)).ok_or(
                        Error::new(ErrorKind::InvalidInput, format!("Invalid feature '{}'",
                                                                    feature_id))
                    )?)
                }
            };

            let size = module.sizes.get(&kind.size).ok_or(
                Error::new(ErrorKind::InvalidInput, format!("Invalid size '{}'", kind.size))
            )?;

            kinds.insert(id, TransitionKind {
                feature,
                feature_offset: kind.feature_offset,
                size: Rc::clone(size),
            });
        }

        Ok(TransitionParams {
            kinds,
            spacing: builder.spacing,
        })
    }
}

pub(crate) struct TransitionKind {
    feature: Option<Rc<Feature>>,
    size: Rc<ObjectSize>,
    feature_offset: Point,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionParamsBuilder {
    spacing: u32,
    kinds: HashMap<String, TransitionKindBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionKindBuilder {
    size: String,
    feature_offset: Point,
    feature: Option<String>,
}
