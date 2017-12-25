use std::io::{Error, ErrorKind};

use image::Image;
use util::Point;
use resource::ResourceBuilder;
use io::TextRenderer;
use ui::{AnimationState, Size};

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug)]
pub struct SimpleImage {
    id: String,
    text_display: Vec<char>,
    size: Size,
}

impl SimpleImage {
    fn validate(resource: SimpleImage) -> Result<SimpleImage, Error> {
        if resource.text_display.len() == 0 {
            return Ok(resource);
        }

        if resource.text_display.len() != (resource.size.product()) as usize {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("SimpleImage text display must be length*width characters.")));
        }

        Ok(resource)
    }
}

impl ResourceBuilder for SimpleImage {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn from_json(data: &str) -> Result<SimpleImage, Error> {
        let resource: SimpleImage = serde_json::from_str(data)?;
        SimpleImage::validate(resource)
    }

    fn from_yaml(data: &str) -> Result<SimpleImage, Error> {
        let resource: Result<SimpleImage, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => SimpleImage::validate(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
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
