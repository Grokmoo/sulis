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
use std::char;
use std::collections::HashMap;

use io::Vertex;
use resource::ResourceBuilder;
use util::{invalid_data_error, Point, Size};
use config::CONFIG;

use serde_json;
use serde_yaml;

use extern_image::{self, ImageBuffer, Rgba};

pub struct Font {
    pub id: String,
    pub line_height: u32,
    pub base: u32,
    characters: HashMap<char, FontChar>,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub struct FontChar {
    size: Size,
    offset: Point,
    x_advance: u32,
    tex_coords: [f32; 8],
}

impl Font {
    pub fn get_width(&self, text: &str) -> i32 {
        let mut width: i32 = 0;
        for c in text.chars() {
            let font_char = match self.characters.get(&c) {
                None => continue,
                Some(font_char) => font_char,
            };

            width += font_char.x_advance as i32;
        }

        width
    }

    /// Adds a quad for the given character to the quads list if the
    /// character can be found in the font.  If not, does nothing.
    /// Returns the position that the next character in the line should
    /// be drawn at (i.e. pos_x plus x_advance for the font character)
    /// `line_height` scales the drawing, 1.0 for no scaling
    pub fn get_quad(&self, quads: &mut Vec<Vertex>, c: char,
                    pos_x: f32, pos_y: f32, line_height: f32) -> f32 {
        let font_char = match self.characters.get(&c) {
            None => return pos_x,
            Some(font_char) => font_char,
        };
        let scale_factor = line_height / self.line_height as f32;

        let tc = &font_char.tex_coords;
        let x_char = pos_x + scale_factor * font_char.offset.x as f32;
        let y_char = pos_y + scale_factor * font_char.offset.y as f32;
        let x_min = x_char;
        let y_min = CONFIG.display.height as f32 - y_char;
        let x_max = x_char + scale_factor * font_char.size.width as f32;
        let y_max = CONFIG.display.height as f32
            - (y_char + scale_factor * font_char.size.height as f32);
        quads.append(&mut vec![
            Vertex { position: [ x_min, y_max ], tex_coords: [tc[0], tc[1]] },
            Vertex { position: [ x_min, y_min ], tex_coords: [tc[2], tc[3]] },
            Vertex { position: [ x_max, y_max ], tex_coords: [tc[4], tc[5]] },
            Vertex { position: [ x_max, y_min ], tex_coords: [tc[6], tc[7]] },
            Vertex { position: [ x_min, y_min ], tex_coords: [tc[2], tc[3]] },
            Vertex { position: [ x_max, y_max ], tex_coords: [tc[4], tc[5]] },
        ]);
        pos_x + scale_factor * (font_char.x_advance as f32)
    }

    pub fn new(dir: &str, builder: FontBuilder) -> Result<Rc<Font>, Error> {
        let filename = format!("{}{}", dir, builder.src);
        let image = match extern_image::open(&filename) {
            Ok(image) => image,
            Err(e) => {
                warn!("Error reading '{}', {}", &filename, e);
                return invalid_data_error(
                    &format!("Cannot open font image at '{}'", filename));
            }
        };
        let image = image.to_rgba();
        let (image_width, image_height) = image.dimensions();
        let image_size = Size::new(image_width as i32, image_height as i32);

        let mut characters: HashMap<char, FontChar> = HashMap::new();
        for char_builder in builder.characters {
            let id = match char::from_u32(char_builder.id) {
                None => return invalid_data_error(
                    &format!("'{}' is not a valid utf8 character.", char_builder.id)),
                Some(c) => c,
            };

            let position = Point::new(char_builder.xywh[0] as i32, char_builder.xywh[1] as i32);
            let size = Size::new(char_builder.xywh[2] as i32, char_builder.xywh[3] as i32);

            let image_width = image_size.width as f32;
            let image_height = image_size.height as f32;
            let x_min = (position.x as f32) / image_width;
            let y_min = (image_height - (position.y + size.height) as f32) / image_height;
            let x_max = (position.x + size.width) as f32 / image_width;
            let y_max = (image_height - position.y as f32) / image_height;
            characters.insert(id, FontChar {
                size,
                offset: char_builder.offset,
                x_advance: char_builder.x_advance,
                tex_coords: [ x_min, y_min,
                              x_min, y_max,
                              x_max, y_min,
                              x_max, y_max ],
            });
        }

        Ok(Rc::new(Font {
            id: builder.id,
            line_height: builder.line_height,
            base: builder.base,
            characters,
            image,
        }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontBuilder {
    id: String,
    src: String,
    line_height: u32,
    base: u32,
    characters: Vec<FontCharBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FontCharBuilder {
    id: u32,
    xywh: [u32; 4],
    offset: Point,
    x_advance: u32
}

impl ResourceBuilder for FontBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<FontBuilder, Error> {
        let resource: FontBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<FontBuilder, Error> {
        let resource: Result<FontBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
