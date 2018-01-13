use std::rc::Rc;
use std::io::{Error, ErrorKind};

use image::Image;
use resource::{ResourceBuilder, ResourceSet, Sprite};
use io::{DrawList, GraphicsRenderer, TextRenderer};
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

        let sprite = resources.get_sprite_internal(&builder.image_display)?;

        Ok(Rc::new(SimpleImage {
            id: builder.id,
            text_display: builder.text_display,
            size: builder.size,
            image_display: sprite,
        }))
    }
}

impl Image for SimpleImage {
    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, _state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32) {
        renderer.draw(DrawList::from_sprite_f32(&self.image_display, x, y, w, h));
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

