use std::rc::Rc;
use std::io::{Error, ErrorKind};

use image::Image;
use util::Point;
use resource::{ResourceBuilder, ResourceSet, Sprite};
use io::{DrawList, TextRenderer, Vertex};
use ui::{AnimationState, Size};
use util::invalid_data_error;
use config::CONFIG;

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
        let tc = &self.image_display.tex_coords;
        let x_min = position.x as f32;
        let y_min = CONFIG.display.height as f32 - position.y as f32;
        let x_max = (position.x + size.width) as f32;
        let y_max = CONFIG.display.height as f32 - (position.y + size.height) as f32;

        DrawList::from_sprite(&self.image_display.id, vec![[
                      Vertex { position: [ x_min, y_max ], tex_coords: [tc[0], tc[1]] },
                      Vertex { position: [ x_min, y_min ], tex_coords: [tc[2], tc[3]] },
                      Vertex { position: [ x_max, y_max ], tex_coords: [tc[4], tc[5]] },
                      Vertex { position: [ x_max, y_min ], tex_coords: [tc[6], tc[7]] },
        ]])
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

