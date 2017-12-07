use io::TextRenderer;

pub struct Cursor {
    pub c: char,
    pub x: u32,
    pub y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

impl Cursor {
    pub fn move_by(&mut self, x: i32, y: i32) -> bool {
        let new_x = (self.x as i32) + x;
        let new_y = (self.y as i32) + y;

        if new_x < 0 || new_y < 0 {
            return false;
        }

        let new_x = new_x as u32;
        let new_y = new_y as u32;
        if new_x >= self.max_x || new_y >= self.max_y {
            return false;
        }

        self.x = new_x;
        self.y = new_y;

        true
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer, millis: u32) {
        if millis % 1_000 < 500 { return; }

        renderer.set_cursor_pos(self.x, self.y);
        renderer.render_char(self.c);
    }
}
