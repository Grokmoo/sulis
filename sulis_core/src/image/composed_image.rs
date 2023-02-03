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

use std::io::{Error, ErrorKind};
use std::rc::Rc;

use crate::image::simple_image::SimpleImageBuilder;
use crate::image::{Image, SimpleImage};
use crate::io::{DrawList, GraphicsRenderer};
use crate::resource::ResourceSet;
use crate::ui::AnimationState;
use crate::util::{invalid_data_error, Rect, Size};

const GRID_DIM: i32 = 3;
const GRID_LEN: i32 = GRID_DIM * GRID_DIM;

#[derive(Debug)]
pub struct ComposedImage {
    images: Vec<Rc<dyn Image>>,
    id: String,
    size: Size,
    middle_size: Size,
}

fn get_images_from_grid(
    grid: Vec<String>,
    resources: &ResourceSet,
) -> Result<Vec<Rc<dyn Image>>, Error> {
    let mut images_vec: Vec<Rc<dyn Image>> = Vec::new();
    for id in grid {
        let image = resources.images.get(&id);
        if image.is_none() {
            return invalid_data_error(&format!("Unable to locate sub image {id}"));
        }

        let image = image.unwrap();
        images_vec.push(Rc::clone(image));
    }

    Ok(images_vec)
}

fn get_images_from_inline(
    grid: Vec<String>,
    sub_image_data: SubImageData,
    resources: &mut ResourceSet,
) -> Result<Vec<Rc<dyn Image>>, Error> {
    let size = sub_image_data.size;
    let spritesheet = sub_image_data.spritesheet;

    let mut images: Vec<Rc<dyn Image>> = Vec::new();
    for id in grid {
        let image_display = format!("{spritesheet}/{id}");
        let builder = SimpleImageBuilder {
            id: id.clone(),
            image_display,
            size,
        };
        let image = SimpleImage::generate(builder, resources)?;
        resources.images.insert(id, Rc::clone(&image));
        images.push(image);
    }

    Ok(images)
}

impl ComposedImage {
    pub fn generate(
        builder: ComposedImageBuilder,
        resources: &mut ResourceSet,
    ) -> Result<Rc<dyn Image>, Error> {
        if builder.grid.len() as i32 != GRID_LEN {
            return invalid_data_error(&format!("Composed image grid must be length {GRID_LEN}"));
        }

        let images_vec = match builder.generate_sub_images {
            Some(sub_image_data) => {
                get_images_from_inline(builder.grid, sub_image_data, resources)?
            }
            None => get_images_from_grid(builder.grid, resources)?,
        };

        // verify heights make sense for the grid
        let mut total_height = 0;
        for y in 0..GRID_DIM {
            let row_height = images_vec
                .get((y * GRID_DIM) as usize)
                .unwrap()
                .get_size()
                .height;

            for x in 0..GRID_DIM {
                let height = images_vec
                    .get((y * GRID_DIM + x) as usize)
                    .unwrap()
                    .get_size()
                    .height;

                if height != row_height {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("All images in row {y} must have the same height"),
                    ));
                }
            }
            total_height += row_height;
        }

        //verify widths make sense for the grid
        let mut total_width = 0;
        for x in 0..GRID_DIM {
            let col_width = images_vec.get(x as usize).unwrap().get_size().width;

            for y in 0..GRID_DIM {
                let width = images_vec
                    .get((y * GRID_DIM + x) as usize)
                    .unwrap()
                    .get_size()
                    .width;

                if width != col_width {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("All images in col {x} must have the same width"),
                    ));
                }
            }
            total_width += col_width;
        }

        let middle_size = *images_vec.get((GRID_LEN / 2) as usize).unwrap().get_size();

        Ok(Rc::new(ComposedImage {
            images: images_vec,
            size: Size::new(total_width, total_height),
            middle_size,
            id: builder.id,
        }))
    }
}

impl Image for ComposedImage {
    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        let fill_width = rect.w - (self.size.width - self.middle_size.width) as f32;
        let fill_height = rect.h - (self.size.height - self.middle_size.height) as f32;

        let image = &self.images[0];
        let mut draw = Rect {
            x: rect.x,
            y: rect.y,
            w: image.get_width_f32(),
            h: image.get_height_f32(),
        };
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[1];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[2];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x = rect.x;
        draw.y += image.get_height_f32();
        let image = &self.images[3];
        draw.w = image.get_width_f32();
        draw.h = fill_height;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[4];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[5];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x = rect.x;
        draw.y += fill_height;
        let image = &self.images[6];
        draw.w = image.get_width_f32();
        draw.h = image.get_height_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += image.get_width_f32();
        let image = &self.images[7];
        draw.w = fill_width;
        image.append_to_draw_list(draw_list, state, draw, millis);

        draw.x += fill_width;
        let image = &self.images[8];
        draw.w = image.get_width_f32();
        image.append_to_draw_list(draw_list, state, draw, millis);
    }

    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        let mut draw_list = DrawList::empty_sprite();
        self.append_to_draw_list(&mut draw_list, state, rect, millis);
        renderer.draw(draw_list);
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

    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct SubImageData {
    size: Size,
    spritesheet: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ComposedImageBuilder {
    id: String,
    grid: Vec<String>,
    generate_sub_images: Option<SubImageData>,
}
