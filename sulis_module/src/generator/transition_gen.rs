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

use serde::Deserialize;

use crate::generator::{overlaps_any, Rect};
use crate::{
    area::{
        tile::{Feature, Tile},
        ToKind, TransitionAreaParams, TransitionBuilder,
    },
    Module, ObjectSize,
};
use sulis_core::util::{Point, ReproducibleRandom};

pub struct TransitionGen<'a> {
    width: i32,
    height: i32,
    params: &'a TransitionParams,
}

impl<'a> TransitionGen<'a> {
    pub(crate) fn new(width: i32, height: i32, params: &'a TransitionParams) -> TransitionGen<'a> {
        TransitionGen {
            width,
            height,
            params,
        }
    }

    pub fn generate(
        &mut self,
        random: &mut ReproducibleRandom,
        transitions: &[TransitionAreaParams],
    ) -> Result<Vec<TransitionOutput>, Error> {
        let mut gened = Vec::new();
        let mut out = Vec::new();
        for transition in transitions {
            let kind = self.params.kinds.get(&transition.kind).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid transition kind '{}'", transition.kind),
                )
            })?;

            // keep trying until we succeed.  we always want to place transitions
            let mut placed = false;
            for _ in 0..1000 {
                let data = TransitionData::gen(
                    random,
                    self.width,
                    self.height,
                    kind.size.width,
                    kind.size.height,
                );

                if overlaps_any(&data, &gened, self.params.spacing as i32) {
                    continue;
                }

                let mut tiles_out = Vec::new();
                if let Some(feature) = &kind.feature {
                    let base_x = data.x + kind.feature_offset.x;
                    let base_y = data.y + kind.feature_offset.y;
                    for (tile, p) in feature.rand_entry() {
                        tiles_out.push((Rc::clone(tile), base_x + p.x, base_y + p.y));
                    }
                }

                let transition_out = TransitionBuilder {
                    from: Point::new(data.x, data.y),
                    size: kind.size.id.to_string(),
                    to: ToKind::FindLink {
                        id: transition.to.to_string(),
                        x_offset: kind.transition_offset.x,
                        y_offset: kind.transition_offset.y,
                    },
                    hover_text: transition.hover_text.to_string(),
                    image_display: "empty".to_string(),
                };
                out.push(TransitionOutput {
                    transition: transition_out,
                    tiles: tiles_out,
                });

                placed = true;
                gened.push(data);
                break;
            }

            if !placed {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unable to place transition",
                ));
            }
        }

        Ok(out)
    }
}

pub struct TransitionOutput {
    pub transition: TransitionBuilder,
    pub tiles: Vec<(Rc<Tile>, i32, i32)>,
}

#[derive(Clone)]
struct TransitionData {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl TransitionData {
    fn gen(
        random: &mut ReproducibleRandom,
        max_x: i32,
        max_y: i32,
        w: i32,
        h: i32,
    ) -> TransitionData {
        // the tile buffer keeps transitions off the edge of the usable space
        let x = random.gen(8, max_x - w - 16);
        let y = random.gen(8, max_y - h - 16);

        TransitionData {
            x,
            y,
            w,
            h,
        }
    }
}

impl Rect for TransitionData {
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

pub(crate) struct TransitionParams {
    spacing: u32,
    kinds: HashMap<String, TransitionKind>,
}

impl TransitionParams {
    pub(crate) fn new(
        builder: TransitionParamsBuilder,
        module: &Module,
    ) -> Result<TransitionParams, Error> {
        let mut kinds = HashMap::new();

        for (id, kind) in builder.kinds {
            let feature = match kind.feature {
                None => None,
                Some(feature_id) => Some(
                    module
                        .features
                        .get(&feature_id)
                        .map(Rc::clone)
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::InvalidInput,
                                format!("Invalid feature '{feature_id}'"),
                            )
                        })?,
                ),
            };

            let kind_size = kind.size;
            let size = module.sizes.get(&kind_size).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid size '{kind_size}'"),
                )
            })?;

            kinds.insert(
                id,
                TransitionKind {
                    feature,
                    feature_offset: kind.feature_offset,
                    transition_offset: kind.transition_offset,
                    size: Rc::clone(size),
                },
            );
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
    transition_offset: Point,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionParamsBuilder {
    spacing: u32,
    kinds: HashMap<String, TransitionKindBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionKindBuilder {
    size: String,
    feature_offset: Point,
    transition_offset: Point,
    feature: Option<String>,
}
