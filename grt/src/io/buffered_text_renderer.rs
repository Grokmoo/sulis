use io::TextRenderer;
use util::{Point, Size};

pub struct BufferedTextRenderer {
    last_buffer: Vec<char>,
    buffer: Vec<char>,
    cursor_pos: Point,
    size: Size,
}

impl BufferedTextRenderer {
    pub fn new(size: Size) -> BufferedTextRenderer {
        BufferedTextRenderer {
            last_buffer: vec![' ';(size.product() as usize)],
            buffer: vec![' '; (size.product() as usize)],
            cursor_pos: Point::as_zero(),
            size: size,
        }
    }

    fn set_char_at_cursor(&mut self, c: char) {
        *self.buffer.get_mut((self.cursor_pos.x + self.cursor_pos.y *
                              self.size.width) as usize).unwrap() = c;
        self.cursor_pos.add_x(1);
    }

    fn cursor_pos_valid(&self) -> bool {
        self.cursor_pos.in_bounds(self.size.width, self.size.height)
    }

    pub fn clear(&mut self) {
        self.last_buffer.iter_mut().for_each(|c| *c = ' ');
    }

    #[allow(dead_code)] // used on windows, not on unix
    pub fn get_line(&self, y: i32) -> String {
        let s = &self.buffer[(y * self.size.width) as usize..
            ((y + 1) * self.size.width) as usize];

        let out: String = s.iter().collect();
        out
    }

    #[allow(dead_code)] // used on unix, not windows
    pub fn get_char(&self, x: i32, y: i32) -> char {
        self.buffer[(x + y * self.size.width) as usize]
    }

    #[allow(dead_code)] // used on unix, not windows
    pub fn has_changed(&self, x: i32, y: i32) -> bool {
        let cur = self.buffer[(x + y * self.size.width) as usize];
        let last = self.last_buffer[(x + y * self.size.width) as usize];

        cur != last
    }

    #[allow(dead_code)] // used on unix, not windows
    pub fn swap(&mut self) {
        self.last_buffer.copy_from_slice(&self.buffer);
    }
}

impl TextRenderer for BufferedTextRenderer {
    fn render_char(&mut self, c: char) {
        if self.cursor_pos_valid() {
            self.set_char_at_cursor(c);
        }
    }

    fn render_string(&mut self, s: &str) {
        for c in s.chars() {
            self.render_char(c);
        }
    }

    fn render_chars(&mut self, cs: &[char]) {
        for c in cs.iter() {
            self.render_char(*c);
        }
    }

    fn set_cursor_pos(&mut self, x: i32, y: i32) {
        self.cursor_pos.set(x, y);
    }
}
