//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::cell::RefCell;
use std::rc::Rc;

use io::{event, Event};
use config::CONFIG;
use ui::Widget;
use util::Point;

pub struct Cursor {
    pub x: i32,
    pub y: i32,
    pub max_x: i32,
    pub max_y: i32,

    pub xf: f32,
    pub yf: f32,
}

thread_local! {
    static CURSOR: RefCell<Cursor> = RefCell::new(Cursor {
        x: 0,
        y: 0,
        max_x: CONFIG.display.width,
        max_y: CONFIG.display.height,
        xf: 0.0,
        yf: 0.0,
    });
}

impl Cursor {
    pub fn move_by(root: &Rc<RefCell<Widget>>, x: f32, y: f32) {
        if !Cursor::move_by_internal(x, y) {
            return;
        }

        trace!("Cursor move by {}, {}", x, y);

        let event = Event::new(event::Kind::MouseMove { change: Point::new(x as i32, y as i32) });
        Widget::dispatch_event(&root, event);
    }

    pub fn move_to(root: &Rc<RefCell<Widget>>, x: f32, y: f32) {
        let (cur_x, cur_y) = Cursor::get_position_f32();
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

    pub fn move_by_internal(x: f32, y: f32) -> bool {
        if x == 0.0 && y == 0.0 {
            return false;
        }

        CURSOR.with(|c| {
            let mut cursor = c.borrow_mut();

            let new_x = cursor.xf + x;
            let new_y = cursor.yf + y;

            if new_x < 0.0 || new_y < 0.0 {
                return false;
            }

            if new_x as i32 >= cursor.max_x || new_y as i32 >= cursor.max_y {
                return false;
            }

            cursor.xf = new_x;
            cursor.yf = new_y;

            cursor.x = new_x as i32;
            cursor.y = new_y as i32;
            true
        })
    }

    pub fn get_position_f32() -> (f32, f32) {
        CURSOR.with(|c| (c.borrow().xf, c.borrow().yf))
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

    pub fn get_x_f32() -> f32 {
        CURSOR.with(|c| c.borrow().xf)
    }

    pub fn get_y_f32() -> f32 {
        CURSOR.with(|c| c.borrow().yf)
    }
}
