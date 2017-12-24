use std::cell::RefCell;

use io::TextRenderer;
use config::CONFIG;

pub struct Cursor {
    pub c: char,
    pub x: i32,
    pub y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

thread_local! {
    static CURSOR: RefCell<Cursor> = RefCell::new(Cursor {
        x: 0,
        y: 0,
        max_x: CONFIG.display.width,
        max_y: CONFIG.display.height,
        c: CONFIG.display.cursor_char,
    });
}

impl Cursor {
    pub fn move_by(x: i32, y: i32) -> bool {
        CURSOR.with(|c| {
            let mut cursor = c.borrow_mut();

            let new_x = (cursor.x as i32) + x;
            let new_y = (cursor.y as i32) + y;

            if new_x < 0 || new_y < 0 {
                return false;
            }

            if new_x >= cursor.max_x || new_y >= cursor.max_y {
                return false;
            }

            cursor.x = new_x;
            cursor.y = new_y;

            true
        })
    }

    pub fn get_position() -> (i32, i32) {
        CURSOR.with(|c| (c.borrow().x, c.borrow().y))
    }

    pub fn get_x() -> i32 {
        CURSOR.with(|c| c.borrow().x)
    }

    pub fn get_y() -> i32 {
        CURSOR.with(|c| c.borrow().y)
    }

    pub fn draw_text_mode(renderer: &mut TextRenderer, millis: u32) {
        if millis % 1_000 < 500 { return; }

        CURSOR.with(|c| {
            let cursor = c.borrow();
            renderer.set_cursor_pos(cursor.x, cursor.y);
            renderer.render_char(cursor.c);
        });
    }
}
