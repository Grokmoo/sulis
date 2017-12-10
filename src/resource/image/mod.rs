pub mod simple_image;
pub use self::simple_image::SimpleImage;

pub mod composed_image;
pub use self::composed_image::ComposedImage;

use io::TextRenderer;

pub trait Image {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, x: i32, y: i32);

    fn fill_text_mode(&self, renderer: &mut TextRenderer, x: i32, y: i32,
                      width: i32, height: i32) {

        let mut rel_y = 0;
        while rel_y < height {
            let mut rel_x = 0;
            while rel_x < width {
                self.draw_text_mode(renderer, x + rel_x, y + rel_y);
                rel_x += self.get_width();
            }
            rel_y += self.get_height();
        }
    }

    fn get_width(&self) -> i32;

    fn get_height(&self) -> i32;
}
