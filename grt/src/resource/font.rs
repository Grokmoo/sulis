use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::char;
use std::collections::HashMap;

use resource::ResourceBuilder;
use ui::Size;
use util::{invalid_data_error, Point};

use serde_json;
use serde_yaml;

use extern_image::{self, ImageBuffer, Rgba};

pub struct Font {
    id: String,
    name: String,
    line_height: u32,
    base: u32,
    characters: HashMap<char, FontChar>,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub struct FontChar {
    id: char,
    position: Point,
    size: Size,
    offset: Point,
    x_advance: u32,
}

impl Font {
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

        let mut characters: HashMap<char, FontChar> = HashMap::new();
        for char_builder in builder.characters {
            let id = match char::from_u32(char_builder.id) {
                None => return invalid_data_error(
                    &format!("'{}' is not a valid utf8 character.", char_builder.id)),
                Some(c) => c,
            };

            let position = Point::new(char_builder.xywh[0] as i32, char_builder.xywh[1] as i32);
            let size = Size::new(char_builder.xywh[2] as i32, char_builder.xywh[3] as i32);

            characters.insert(id, FontChar {
                id,
                position,
                size,
                offset: char_builder.offset,
                x_advance: char_builder.x_advance,
            });
        }

        Ok(Rc::new(Font {
            id: builder.id,
            name: builder.name,
            line_height: builder.line_height,
            base: builder.base,
            characters,
            image,
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct FontBuilder {
    id: String,
    name: String,
    src: String,
    line_height: u32,
    base: u32,
    characters: Vec<FontCharBuilder>,
}

#[derive(Debug, Deserialize)]
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
