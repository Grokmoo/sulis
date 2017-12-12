use std::io::{Error, ErrorKind};

use resource::{Image, Point, ResourceBuilder};
use io::TextRenderer;
use ui::Size;

use serde_json;

#[derive(Deserialize, Debug)]
pub struct SimpleImage {
    id: String,
    text_display: Vec<char>,
    size: Size,
}

impl ResourceBuilder for SimpleImage {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn new(data: &str) -> Result<SimpleImage, Error> {
        let image: SimpleImage = serde_json::from_str(data)?;

        if image.text_display.len() != (image.size.product()) as usize {
            return Err(Error::new(ErrorKind::InvalidData,
                format!("SimpleImage text display must be length*width characters.")));
        }

        Ok(image)
    }
}

impl Image for SimpleImage {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, _state: &str, position: &Point) {
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
