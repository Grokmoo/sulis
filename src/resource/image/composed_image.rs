use std::rc::Rc;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

use resource::{Image, ResourceBuilder};
use io::TextRenderer;

use serde_json;

const GRID_DIM: i32 = 3;
const GRID_LEN: i32 = GRID_DIM * GRID_DIM;

pub struct ComposedImage {
    images: Vec<Rc<Image>>,

    width: i32,
    height: i32,
    middle_width: i32,
    middle_height: i32,
}

impl ComposedImage {
    pub fn new(builder: ComposedImageBuilder,
               images: &HashMap<String, Rc<Image>>) -> Result<Rc<Image>, Error> {
        if builder.grid.len() as i32 != GRID_LEN {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("Composed image grid must be length {}", GRID_LEN)));
        }

        let mut images_vec: Vec<Rc<Image>> = Vec::new();
        for id in builder.grid {
           let image = images.get(&id);
           if let None = image {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Unable to locate sub image {}", id)));
           }

           let image = image.unwrap();
           images_vec.push(Rc::clone(image));
        }

        // verify heights make sense for the grid
        let mut total_height = 0;
        for y in 0..GRID_DIM {
            let row_height = images_vec.get((y * GRID_DIM) as usize)
                .unwrap().get_height();

            for x in 0..GRID_DIM {
                let height = images_vec.get((y * GRID_DIM + x) as usize)
                    .unwrap().get_height();

                if height != row_height {
                    return Err(Error::new(ErrorKind::InvalidData,
                         format!("All images in row {} must have the same height", y)));
                }
            }
            total_height += row_height;
        }

        //verify widths make sense for the grid
        let mut total_width = 0;
        for x in 0..GRID_DIM {
            let col_width = images_vec.get(x as usize).unwrap().get_width();

            for y in 0..GRID_DIM {
                let width = images_vec.get((y * GRID_DIM + x) as usize)
                    .unwrap().get_width();

                if width != col_width {
                    return Err(Error::new(ErrorKind::InvalidData,
                        format!("All images in col {} must have the same width", x)));
                }
            }
            total_width += col_width;
        }

        let middle_width = images_vec.get((GRID_LEN / 2) as usize)
            .unwrap().get_width();
        let middle_height = images_vec.get((GRID_LEN / 2) as usize)
            .unwrap().get_height();

        Ok(Rc::new(ComposedImage {
            images: images_vec,
            width: total_width,
            height: total_height,
            middle_width,
            middle_height,
        }))
    }
}

impl Image for ComposedImage {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, x: i32, y: i32) {
        renderer.set_cursor_pos(x, y);

        let mut cur_x = x;
        let mut cur_y = y;
        for (index, image) in self.images.iter().enumerate() {
            let index = index as i32;
            image.draw_text_mode(renderer, cur_x, cur_y);

            if index % GRID_DIM == GRID_DIM - 1 {
                cur_x = x;
                cur_y += image.get_height();
            }
        }
    }

    //// Renders text for this composed image to the given coordinates.
    //// This method assumes that 'GRID_DIM' equals 3 for simplicity
    //// and performance purposes.
    fn fill_text_mode(&self, renderer: &mut TextRenderer, x: i32, y: i32,
                      width: i32, height: i32) {
        let fill_width = width - (self.width - self.middle_width);
        let fill_height = height - (self.height - self.middle_height);

        unsafe {
            let image = self.images.get_unchecked(0);
            image.draw_text_mode(renderer, x, y);

            let cur_x = x + image.get_width();
            let image = self.images.get_unchecked(1);
            image.fill_text_mode(renderer, cur_x, y, fill_width, image.get_height());

            let cur_x = cur_x + fill_width;
            let image = self.images.get_unchecked(2);
            image.draw_text_mode(renderer, cur_x, y);

            let cur_y = y + image.get_height();
            let cur_x = x;
            let image = self.images.get_unchecked(3);
            image.fill_text_mode(renderer, cur_x, cur_y, image.get_width(), fill_height);

            let cur_x = cur_x + image.get_width();
            let image = self.images.get_unchecked(4);
            image.fill_text_mode(renderer, cur_x, cur_y, fill_width, fill_height);

            let cur_x = cur_x + fill_width;
            let image = self.images.get_unchecked(5);
            image.fill_text_mode(renderer, cur_x, cur_y, image.get_width(), fill_height);

            let cur_x = x;
            let cur_y = cur_y + fill_height;
            let image = self.images.get_unchecked(6);
            image.draw_text_mode(renderer, cur_x, cur_y);

            let cur_x = cur_x + image.get_width();
            let image = self.images.get_unchecked(7);
            image.fill_text_mode(renderer, cur_x, cur_y, fill_width, image.get_height());

            let cur_x = cur_x + fill_width;
            let image = self.images.get_unchecked(8);
            image.draw_text_mode(renderer, cur_x, cur_y);
        }
    }

    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposedImageBuilder {
    pub id: String,
    pub grid: Vec<String>,
}

impl ResourceBuilder for ComposedImageBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<ComposedImageBuilder, Error> {
        let image: ComposedImageBuilder = serde_json::from_str(data)?;

        Ok(image)
    }
}
