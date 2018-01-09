use std::rc::Rc;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

use serde_json;
use serde_yaml;

use image::Image;
use resource::ResourceBuilder;
use io::{GraphicsRenderer, TextRenderer};
use ui::AnimationState;
use util::{Point, Size};

#[derive(Debug)]
pub struct AnimatedImage {
    images: HashMap<AnimationState, Rc<Image>>,

    size: Size,
}

impl AnimatedImage {
    pub fn new(builder: AnimatedImageBuilder,
               images: &HashMap<String, Rc<Image>>) -> Result<Rc<Image>, Error> {
        let mut images_map: HashMap<AnimationState, Rc<Image>> = HashMap::new();

        let mut size: Option<Size> = None;
        for (state_str, image_id) in builder.states {
            // check that the state string exists
            let state = AnimationState::parse(&state_str)?;

            let image = images.get(&image_id);
            if let None = image {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Unable to locate sub image {}", image_id)));
            }

            let image = image.unwrap();
            images_map.insert(state, Rc::clone(image));

            if let None = size {
                size = Some(*image.get_size());
            } else {
                if size.unwrap() != *image.get_size() {
                    return Err(Error::new(ErrorKind::InvalidData,
                        format!("All images in an animated image must have the same size.")));
                }
            }
        }

        Ok(Rc::new(AnimatedImage {
            images: images_map,
            size: size.unwrap(),
        }))
    }
}

impl Image for AnimatedImage {
    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32) {
        AnimationState::find_match(&self.images, state).draw_graphics_mode(renderer, state, x, y, w, h);
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      state: &AnimationState, position: &Point) {
        AnimationState::find_match(&self.images, state)
            .draw_text_mode(renderer, state, position);
    }

    fn fill_text_mode(&self, renderer: &mut TextRenderer, state: &AnimationState,
                      position: &Point, size: &Size) {
        AnimationState::find_match(&self.images, state)
            .fill_text_mode(renderer, state, position, size);
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
pub struct AnimatedImageBuilder {
    pub id: String,
    pub states: HashMap<String, String>,
}

impl ResourceBuilder for AnimatedImageBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<AnimatedImageBuilder, Error> {
        let resource: AnimatedImageBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<AnimatedImageBuilder, Error> {
        let resource: Result<AnimatedImageBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
