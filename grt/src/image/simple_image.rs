use std::rc::Rc;
use std::io::{Error, ErrorKind};

use image::Image;
use resource::{ResourceBuilder, ResourceSet, Sprite};
use io::{DrawList, TextRenderer};
use ui::AnimationState;
use util::{invalid_data_error, Point, Size};

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
                return invalid_data_error("SimpleImage text display must be \
                                          length*width characters.");
            }
        }

        let sprite = resources.get_sprite(&builder.image_display)?;

        Ok(Rc::new(SimpleImage {
            id: builder.id,
            text_display: builder.text_display,
            size: builder.size,
            image_display: sprite,
        }))
    }
}

impl Image for SimpleImage {
    fn get_draw_list(&self, _state: &AnimationState, position: &Point, size: &Size) -> DrawList {
        DrawList::from_sprite(&self.image_display, position.x, position.y, size.width, size.height)
    }

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

