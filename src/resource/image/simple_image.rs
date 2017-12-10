use std::io::{Error, ErrorKind};

use resource::{Image, ResourceBuilder};
use io::TextRenderer;

use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleImage {
    id: String,
    text_display: Vec<char>,
    width: i32,
    height: i32,
}

impl ResourceBuilder for SimpleImage {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn new(data: &str) -> Result<SimpleImage, Error> {
        let image: SimpleImage = serde_json::from_str(data)?;

        if image.text_display.len() != (image.width * image.height) as usize {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("SimpleImage text display must be length*width characters.")));
        }

        Ok(image)
    }
}

impl Image for SimpleImage {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, x: i32, y: i32) {
        renderer.set_cursor_pos(x, y);

        for y_rel in 0..self.height {
            renderer.set_cursor_pos(x, y + y_rel);
            let start = (y_rel * self.height) as usize;
            let end = start + self.width as usize;
            renderer.render_chars(&self.text_display[start..end]);
        }
    }

    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }
}
