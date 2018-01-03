use std::cell::RefCell;
use std::rc::Rc;

use io::{event, Event, TextRenderer};
use config::CONFIG;
use ui::Widget;
use util::Point;

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
    pub fn move_by(root: &Rc<RefCell<Widget>>, x: i32, y: i32) {
        trace!("Cursor move by {}, {}", x, y);
        if !Cursor::move_by_internal(x, y) {
            return;
        }

        let event = Event::new(event::Kind::MouseMove { change: Point::new(x, y) });
        Widget::dispatch_event(&root, event);
    }

    pub fn move_to(root: &Rc<RefCell<Widget>>, x: i32, y: i32) {
        trace!("Moving cursor to {}, {}", x, y);
        let (cur_x, cur_y) = Cursor::get_position();
        Cursor::move_by(root, x - cur_x, y - cur_y);
    }

    pub fn press(root: &Rc<RefCell<Widget>>, kind: event::ClickKind) {
        let (x, y) = Cursor::get_position();

        trace!("Cursor pressed at {},{}", x, y);
        let event = Event::new(event::Kind::MousePress(kind));
        Widget::dispatch_event(&root, event);
    }

    pub fn release(root: &Rc<RefCell<Widget>>, kind: event::ClickKind) {
        let (x, y) = Cursor::get_position();

        trace!("Cursor released at {},{}", x, y);
        let event = Event::new(event::Kind::MouseRelease(kind));
        Widget::dispatch_event(&root, event);
    }

    pub fn move_by_internal(x: i32, y: i32) -> bool {
        if x == 0 && y == 0 {
            return false;
        }

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
