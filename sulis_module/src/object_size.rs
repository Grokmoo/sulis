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

use std::io::{Error};
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::util::{invalid_data_error, unable_to_create_error, Point};

#[derive(Debug)]
pub struct ObjectSize {
    pub id: String,
    pub width: i32,
    pub height: i32,
    pub diagonal: f32,
    pub cursor_sprite: Rc<Sprite>,
    pub selection_image: Rc<Image>,
    relative_points: Vec<Point>,
}

impl ObjectSize {
    pub fn new(builder: ObjectSizeBuilder) -> Result<ObjectSize, Error> {
        let mut points: Vec<Point> = Vec::new();

        let width = builder.width as i32;
        let height = builder.height as i32;
        for p in builder.relative_points.into_iter() {
            if p.len() != 2 {
                return invalid_data_error("Point array length must be equal to 2");
            }

            let x = p[0] as i32;
            let y = p[1] as i32;
            if x < 0 || y < 0 || x >= width || y >= height {
                return invalid_data_error(
                    &format!("Point coords must be within {},{}", builder.width, builder.height));
            }

            points.push(Point::new(x, y));
        }

        let sprite = ResourceSet::get_sprite(&builder.cursor_image)?;

        let selection_image = match ResourceSet::get_image(&builder.selection_image) {
            None => {
                warn!("Unable to locate image '{}'", builder.selection_image);
                return unable_to_create_error("object_size", &builder.id);
            }, Some(img) => img,
        };

        let diagonal = (((width * width) + (height * height)) as f32).sqrt();

        Ok(ObjectSize {
            id: builder.id,
            width,
            height,
            diagonal,
            cursor_sprite: sprite,
            selection_image,
            relative_points: points,
        })
    }

    pub fn relative_points(&self) -> ObjectSizeIterator {
        ObjectSizeIterator { size: &self, index: 0, x_offset: 0, y_offset: 0 }
    }

    pub fn points(&self, x: i32, y: i32) -> ObjectSizeIterator {
        ObjectSizeIterator { size: &self, index: 0, x_offset: x, y_offset: y }
    }
}

pub struct ObjectSizeIterator<'a> {
    size: &'a ObjectSize,
    index: usize,
    x_offset: i32,
    y_offset: i32,
}

impl<'a> Iterator for ObjectSizeIterator<'a> {
    type Item = Point;
    fn next(&mut self) -> Option<Point> {
        let next = self.size.relative_points.get(self.index);

        self.index += 1;

        match next {
            None => None,
            Some(p) => Some(p.add(self.x_offset, self.y_offset))
        }
    }
}

impl PartialEq for ObjectSize {
    fn eq(&self, other: &ObjectSize) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjectSizeBuilder {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub cursor_image: String,
    pub selection_image: String,
    pub relative_points: Vec<Vec<usize>>,
}
