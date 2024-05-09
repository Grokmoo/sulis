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

use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use serde::Deserialize;

use crate::image::Image;
use crate::io::{DrawList, GraphicsRenderer};
use crate::ui::AnimationState;
use crate::util::{invalid_data_error, Rect, Size};

#[derive(Debug)]
pub struct AnimatedImage {
    id: String,
    images: Vec<(AnimationState, Rc<dyn Image>)>,
    size: Size,
}

impl AnimatedImage {
    pub fn generate(
        builder: AnimatedImageBuilder,
        images: &HashMap<String, Rc<dyn Image>>,
    ) -> Result<Rc<dyn Image>, Error> {
        let mut images_vec = Vec::new();

        if builder.states.is_empty() {
            return invalid_data_error("Animated image must have 1 or more sub images.");
        }

        let mut size: Option<Size> = None;
        for (state_str, image_id) in builder.states {
            // check that the state string exists
            let state = AnimationState::parse(&state_str)?;

            let image = match images.get(&image_id) {
                None => {
                    return invalid_data_error(&format!(
                        "Unable to locate sub \
                         image '{image_id}'"
                    ));
                }
                Some(image) => Rc::clone(image),
            };

            match size {
                None => size = Some(*image.get_size()),
                Some(ref size) => {
                    if *size != *image.get_size() {
                        return invalid_data_error(
                            "All images in an animated image \
                             must have the same size.",
                        );
                    }
                }
            }

            images_vec.push((state, image));
        }

        images_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        // images_vec.sort_unstable_by_key(|&(ref state, _)| state);

        Ok(Rc::new(AnimatedImage {
            images: images_vec,
            size: size.unwrap(),
            id: builder.id,
        }))
    }
}

impl Image for AnimatedImage {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        AnimationState::find_match_in_vec(state, &self.images).draw(renderer, state, rect, millis);
    }

    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        AnimationState::find_match_in_vec(state, &self.images)
            .append_to_draw_list(draw_list, state, rect, millis);
    }

    fn get_width_f32(&self) -> f32 {
        self.size.width as f32
    }

    fn get_height_f32(&self) -> f32 {
        self.size.height as f32
    }

    fn get_size(&self) -> &Size {
        &self.size
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AnimatedImageBuilder {
    pub id: String,
    pub states: HashMap<String, String>,
}
