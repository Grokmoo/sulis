use std::io::{Error, ErrorKind};
use std::rc::Rc;

use grt::resource::{ResourceBuilder, ResourceSet, Sprite};
use grt::util::Point;
use grt::serde_json;
use grt::serde_yaml;

pub struct EntitySize {
    pub size: i32,
    pub cursor_sprite: Rc<Sprite>,
    relative_points: Vec<Point>,
}

impl EntitySize {
    pub fn new(builder: EntitySizeBuilder) -> Result<EntitySize, Error> {
        let mut points: Vec<Point> = Vec::new();

        for p in builder.relative_points.into_iter() {
            if p.len() != 2 {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Point array length is not equal to 2."));
            }
            let x = *p.get(0).unwrap();
            let y = *p.get(1).unwrap();
            if x >= builder.size || y >= builder.size {
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Point has coordinate greater than size '{}'",
                                              builder.size)));
            }

            points.push(Point::new(x as i32, y as i32));
        }

        let sprite = ResourceSet::get_sprite(&builder.cursor_image)?;

        Ok(EntitySize {
            size: builder.size as i32,
            cursor_sprite: sprite,
            relative_points: points,
        })
    }

    pub fn relative_points(&self) -> EntitySizeIterator {
        EntitySizeIterator { size: &self, index: 0, x_offset: 0, y_offset: 0 }
    }

    pub fn points(&self, x: i32, y: i32) -> EntitySizeIterator {
        EntitySizeIterator { size: &self, index: 0, x_offset: x, y_offset: y }
    }
}

pub struct EntitySizeIterator<'a> {
    size: &'a EntitySize,
    index: usize,
    x_offset: i32,
    y_offset: i32,
}

impl<'a> Iterator for EntitySizeIterator<'a> {
    type Item = Point;
    fn next(&mut self) -> Option<Point> {
        let next = self.size.relative_points.get(self.index);

        self.index += 1;

        match next {
            None => None,
            Some(p) => Some(p.add(self.x_offset, self.y_offset))
        }
    }
}

impl PartialEq for EntitySize {
    fn eq(&self, other: &EntitySize) -> bool {
        self.size == other.size
    }
}

#[derive(Deserialize, Debug)]
pub struct EntitySizeBuilder {
    pub size: usize,
    pub cursor_image: String,
    pub relative_points: Vec<Vec<usize>>,
}

impl ResourceBuilder for EntitySizeBuilder {
    fn owned_id(&self) -> String {
        self.size.to_string()
    }

    fn from_json(data: &str) -> Result<EntitySizeBuilder, Error> {
        let resource: EntitySizeBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<EntitySizeBuilder, Error> {
        let resource: Result<EntitySizeBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}

