//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
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

use std::io::Error;

use serde::Deserialize;

use crate::{Module, OnTrigger};

pub struct Frame {
    pub text: String,
}

pub struct Cutscene {
    pub id: String,
    pub frames: Vec<Frame>,
    pub on_end: Vec<OnTrigger>,
}

impl Cutscene {
    pub fn new(builder: CutsceneBuilder, _module: &Module) -> Result<Cutscene, Error> {
        let mut frames = Vec::new();
        for frame_builder in builder.frames {
            let frame = Frame {
                text: frame_builder.text,
            };
            frames.push(frame);
        }

        Ok(Cutscene {
            id: builder.id,
            frames,
            on_end: builder.on_end,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FrameBuilder {
    pub text: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CutsceneBuilder {
    pub id: String,
    pub frames: Vec<FrameBuilder>,

    #[serde(default)]
    pub on_end: Vec<OnTrigger>,
}
