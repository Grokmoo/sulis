use std::rc::Rc;
use std::io::{Error, ErrorKind};

use image::Image;
use util::Point;
use resource::{ResourceBuilder, ResourceSet, Sprite};
use io::TextRenderer;
use ui::{AnimationState, Size};

use serde_json;
use serde_yaml;

#[derive(Debug)]
pub struct SimpleImage {
    id: String,
    text_display: Vec<char>,
    image_display: Rc<Sprite>,
    size: Size,
}

impl SimpleImage {
    pub fn new(builder: SimpleImageBuilder, resources: &ResourceSet) -> Result<Rc<Image>, Error> {
        if builder.text_display.len() != 0 {
            if builder.text_display.len() != (builder.size.product()) as usize {
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("SimpleImage text display must be length*width characters.")));
            }
        }

        let format_error = Err(Error::new(ErrorKind::InvalidData,
                            "SimpleImage image display must be of format {SHEET_ID}/{SPRITE_ID}"));

        let split_index = match builder.image_display.find('/') {
            None => return format_error,
            Some(index) => index,
        };

        let (spritesheet_id, sprite_id) = builder.image_display.split_at(split_index);
        if sprite_id.len() == 0 {
            return format_error;
        }
        let sprite_id = &sprite_id[1..];

        let sheet = match resources.spritesheets.get(spritesheet_id) {
            None => return Err(Error::new(ErrorKind::InvalidData,
                                          format!("Unable to location spritesheet '{}'", spritesheet_id))),
            Some(sheet) => sheet,
        };

        let sprite = match sheet.sprites.get(sprite_id) {
            None => return Err(Error::new(ErrorKind::InvalidData,
                                          format!("Unable to location sprite '{}' in spritesheet '{}'",
                                                  sprite_id, spritesheet_id))),
            Some(ref sprite) => Rc::clone(sprite),
        };

        Ok(Rc::new(SimpleImage {
            id: builder.id,
            text_display: builder.text_display,
            size: builder.size,
            image_display: sprite,
        }))
    }
}

impl Image for SimpleImage {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, _state: &AnimationState,
                      position: &Point) {
        if self.text_display.len() == 0 { return; }

        let x = position.x;
        let y = position.y;

        renderer.set_cursor_pos(x, y);

        for y_rel in 0..self.size.height {
            renderer.set_cursor_pos(x, y + y_rel);
            let start = (y_rel * self.size.height) as usize;
            let end = start + self.size.height as usize;
            renderer.render_chars(&self.text_display[start..end]);
        }
    }

    fn get_size(&self) -> &Size {
        &self.size
    }
}

#[derive(Deserialize, Debug)]
pub struct SimpleImageBuilder {
    id: String,
    text_display: Vec<char>,
    image_display: String,
    size: Size,
}

impl ResourceBuilder for SimpleImageBuilder {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn from_json(data: &str) -> Result<SimpleImageBuilder, Error> {
        let resource: SimpleImageBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<SimpleImageBuilder, Error> {
        let resource: Result<SimpleImageBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}

