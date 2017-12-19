use std::rc::Rc;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

use serde_json;
use serde_yaml;

use resource::{Image, Point, ResourceBuilder};
use io::TextRenderer;
use ui::{AnimationState, Size};

#[derive(Debug)]
pub struct AnimatedImage {
    images: HashMap<String, Rc<Image>>,

    size: Size,
}

impl AnimatedImage {
    pub fn new(builder: AnimatedImageBuilder,
               images: &HashMap<String, Rc<Image>>) -> Result<Rc<Image>, Error> {
        let mut images_map: HashMap<String, Rc<Image>> = HashMap::new();

        let mut size: Option<Size> = None;
        for (state_str, image_id) in builder.states {
            // check that the state string exists
            let state = AnimationState::find(&state_str);
            if let None = state {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Attempted to set non-existant state '{}'", state_str)));
            }

            let image = images.get(&image_id);
            if let None = image {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Unable to locate sub image {}", image_id)));
            }

            let image = image.unwrap();
            images_map.insert(state_str, Rc::clone(image));

            if let None = size {
                size = Some(*image.get_size());
            } else {
                if size.unwrap() != *image.get_size() {
                    return Err(Error::new(ErrorKind::InvalidData,
                        format!("All images in an animated image must have the same size.")));
                }
            }
        }

        // check that all states are accounted for
        for state in AnimationState::iter() {
            let entry = images_map.get(state.get_text());

            if let None = entry {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("AnimatedImage must be specified for state '{}'", state.get_text())));
            }
        }

        if images_map.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("Cannot have an empty animated image.")));
        }

        Ok(Rc::new(AnimatedImage {
            images: images_map,
            size: size.unwrap(),
        }))
    }
}

impl Image for AnimatedImage {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, state: &str, position: &Point) {
        self.images.get(state).unwrap().draw_text_mode(renderer, state, position);
    }

    fn fill_text_mode(&self, renderer: &mut TextRenderer, state: &str,
                      position: &Point, size: &Size) {
        self.images.get(state).unwrap().fill_text_mode(renderer, state, position, size);
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
