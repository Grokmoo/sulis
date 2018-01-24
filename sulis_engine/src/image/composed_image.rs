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

use std::rc::Rc;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

use image::Image;
use resource::ResourceBuilder;
use io::{self, GraphicsRenderer};
use ui::AnimationState;
use util::Size;

use serde_json;
use serde_yaml;

const GRID_DIM: i32 = 3;
const GRID_LEN: i32 = GRID_DIM * GRID_DIM;

#[derive(Debug)]
pub struct ComposedImage {
    images: Vec<Rc<Image>>,

    size: Size,
    middle_size: Size,
}

impl ComposedImage {
    pub fn new(builder: ComposedImageBuilder,
               images: &HashMap<String, Rc<Image>>) -> Result<Rc<Image>, Error> {
        if builder.grid.len() as i32 != GRID_LEN {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("Composed image grid must be length {}", GRID_LEN)));
        }

        let mut images_vec: Vec<Rc<Image>> = Vec::new();
        for id in builder.grid {
           let image = images.get(&id);
           if let None = image {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Unable to locate sub image {}", id)));
           }

           let image = image.unwrap();
           images_vec.push(Rc::clone(image));
        }

        // verify heights make sense for the grid
        let mut total_height = 0;
        for y in 0..GRID_DIM {
            let row_height = images_vec.get((y * GRID_DIM) as usize)
                .unwrap().get_size().height;

            for x in 0..GRID_DIM {
                let height = images_vec.get((y * GRID_DIM + x) as usize)
                    .unwrap().get_size().height;

                if height != row_height {
                    return Err(Error::new(ErrorKind::InvalidData,
                         format!("All images in row {} must have the same height", y)));
                }
            }
            total_height += row_height;
        }

        //verify widths make sense for the grid
        let mut total_width = 0;
        for x in 0..GRID_DIM {
            let col_width = images_vec.get(x as usize).unwrap().get_size().width;

            for y in 0..GRID_DIM {
                let width = images_vec.get((y * GRID_DIM + x) as usize)
                    .unwrap().get_size().width;

                if width != col_width {
                    return Err(Error::new(ErrorKind::InvalidData,
                        format!("All images in col {} must have the same width", x)));
                }
            }
            total_width += col_width;
        }

        let middle_size = *images_vec.get((GRID_LEN / 2) as usize).unwrap().get_size();

        Ok(Rc::new(ComposedImage {
            images: images_vec,
            size: Size::new(total_width, total_height),
            middle_size,
        }))
    }

    fn get_border_image_w(&self, image: &Rc<Image>) -> f32 {
        image.get_width_f32() - io::GFX_BORDER_SCALE
    }

    fn get_border_image_h(&self, image: &Rc<Image>) -> f32 {
        image.get_height_f32() - io::GFX_BORDER_SCALE
    }
}

impl Image for ComposedImage {
    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32) {
        let fill_width = 2.0 * io::GFX_BORDER_SCALE
            + w - (self.size.width - self.middle_size.width) as f32;
        let fill_height = 2.0 * io::GFX_BORDER_SCALE
            + h - (self.size.height - self.middle_size.height) as f32;

        let image = &self.images[0];
        let mut draw_x = x;
        let mut draw_y = y;
        let mut draw_w = self.get_border_image_w(image);
        let mut draw_h = self.get_border_image_h(image);
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += self.get_border_image_w(image);
        let image = &self.images[1];
        draw_w = fill_width;
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += fill_width;
        let image = &self.images[2];
        draw_w = self.get_border_image_w(image);
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x = x;
        draw_y += self.get_border_image_h(image);
        let image = &self.images[3];
        draw_w = self.get_border_image_w(image);
        draw_h = fill_height;
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += self.get_border_image_w(image);
        let image = &self.images[4];
        draw_w = fill_width;
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += fill_width;
        let image = &self.images[5];
        draw_w = self.get_border_image_w(image);
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x = x;
        draw_y += fill_height;
        let image = &self.images[6];
        draw_w = self.get_border_image_w(image);
        draw_h = self.get_border_image_h(image);
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += self.get_border_image_w(image);
        let image = &self.images[7];
        draw_w = fill_width;
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);

        draw_x += fill_width;
        let image = &self.images[8];
        draw_w = self.get_border_image_w(image);
        image.draw_graphics_mode(renderer, state, draw_x, draw_y, draw_w, draw_h);
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
pub struct ComposedImageBuilder {
    pub id: String,
    pub grid: Vec<String>,
}

impl ResourceBuilder for ComposedImageBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<ComposedImageBuilder, Error> {
        let resource: ComposedImageBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<ComposedImageBuilder, Error> {
        let resource: Result<ComposedImageBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
