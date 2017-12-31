use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::ResourceBuilder;
use ui::Size;
use util::Point;

use serde_json;
use serde_yaml;

use extern_image::{self, ImageBuffer, Rgba};

pub struct Spritesheet {
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    sprites: HashMap<String, Sprite>,
}

struct Sprite {
    position: Point,
    size: Size,
}

impl Sprite {
    fn new(position: Point, size: Size) -> Sprite {
        Sprite {
            position,
            size
        }
    }
}

impl Spritesheet {
    pub fn new(dir: &str, builder: SpritesheetBuilder) -> Result<Rc<Spritesheet>, Error> {
        let filename = format!("{}{}", dir, builder.src);
        let image = match extern_image::open(&filename) {
            Ok(image) => image,
            Err(e) => {
                warn!("Error reading '{}', {}", &filename, e);
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Cannot open spritesheet at '{}'", filename)));
            }
        };

        let image = image.to_rgba();
        let (image_width, image_height) = image.dimensions();

        let mut sprites: HashMap<String, Sprite> = HashMap::new();
        for (_id, group) in builder.groups {
            let size = group.size;
            let base_pos = group.position;
            for (id, area_pos) in group.areas {
                let sprite = Sprite::new(area_pos + base_pos, size);

                if sprites.contains_key(&id) {
                    warn!("Duplicate sprite ID in sheet '{}': '{}'", builder.id, id);
                    continue;
                }

                let upper_bound_pos = sprite.position.add(sprite.size.width, sprite.size.height);

                if !sprite.position.in_bounds(image_width as i32, image_height as i32) ||
                    !upper_bound_pos.in_bounds(image_width as i32, image_height as i32) {
                        warn!("Sprite '{}' in sheet '{}' coordinates fall outside image bounds", id, builder.id);
                        continue;
                    }

                sprites.insert(id, sprite);
            }
        }

        Ok(Rc::new(Spritesheet {
            image,
            sprites,
        }))
    }
}

#[derive(Deserialize, Debug)]
pub struct SpritesheetBuilder {
    pub id: String,
    pub src: String,
    pub size: Size,
    groups: HashMap<String, SpritesheetGroup>,
}

#[derive(Deserialize, Debug)]
struct SpritesheetGroup {
    pub size: Size,
    pub position: Point,
    pub areas: HashMap<String, Point>,
}

impl ResourceBuilder for SpritesheetBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<SpritesheetBuilder, Error> {
        let resource: SpritesheetBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<SpritesheetBuilder, Error> {
        let resource: Result<SpritesheetBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
