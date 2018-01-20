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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::rc::Rc;
use std::cell::{RefCell, Ref};

use pancurses;

use io::buffered_text_renderer::BufferedTextRenderer;
use io;
use io::keyboard_event::Key;
use io::{InputAction, IO, KeyboardEvent};
use ui::{Cursor, Widget};
use util::Size;
use config::CONFIG;

pub struct Terminal {
    window: pancurses::Window,
    renderer: BufferedTextRenderer,
    size: Size,
}

impl Terminal {
    pub fn new() -> Terminal {
        debug!("Initialize Pancurses display adapter.");
        let window = pancurses::initscr();
        window.nodelay(true);
        window.keypad(true);
        pancurses::noecho();
        pancurses::curs_set(0);
        pancurses::nonl();

        Terminal::size_terminal();

        let size = Size::new(CONFIG.display.width, CONFIG.display.height);

        Terminal {
            window,
            size,
            renderer: BufferedTextRenderer::new(size),
        }
    }

    #[cfg(target_os = "windows")]
    fn size_terminal() {
        pancurses::resize_term(CONFIG.display.height as i32,
                               CONFIG.display.width as i32);
    }

    #[cfg(not(target_os = "windows"))]
    fn size_terminal() {
        // do nothing
    }
}

impl IO for Terminal {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>) {
        let input = self.window.getch();
        if let None = input {
            return;
        }

        let input = match input.unwrap() {
            pancurses::Input::Character(c) => io::match_char(c),
            input => match_special(input),
        };
        let input = KeyboardEvent { key: input };

        InputAction::handle_action(CONFIG.get_input_action(Some(input)), Rc::clone(&root));
    }

    fn render_output(&mut self, root: Ref<Widget>, millis: u32) {
        self.window.erase();
        self.renderer.clear();

        root.draw_text_mode(&mut self.renderer, millis);

        Cursor::draw_text_mode(&mut self.renderer, millis);

        for y in 0..self.size.height {
            self.window.mv(y, 0);
            self.window.addstr(&self.renderer.get_line(y));
        }
    }
}

fn match_special(c: pancurses::Input) -> Key {
    use io::keyboard_event::Key::*;
    match c {
            pancurses::Input::KeyHome => KeyHome,
            pancurses::Input::KeyEnd => KeyEnd,
            pancurses::Input::KeyIC => KeyInsert,
            pancurses::Input::KeyDC => KeyDelete,
            pancurses::Input::KeyPPage => KeyPageUp,
            pancurses::Input::KeyNPage => KeyPageDown,
            pancurses::Input::KeyUp => KeyUp,
            pancurses::Input::KeyDown => KeyDown,
            pancurses::Input::KeyRight => KeyRight,
            pancurses::Input::KeyLeft => KeyLeft,
            pancurses::Input::KeyF1 => KeyF1,
            pancurses::Input::KeyF2 => KeyF2,
            pancurses::Input::KeyF3 => KeyF3,
            pancurses::Input::KeyF4 => KeyF4,
            pancurses::Input::KeyF5 => KeyF5,
            pancurses::Input::KeyF6 => KeyF6,
            pancurses::Input::KeyF7 => KeyF7,
            pancurses::Input::KeyF8 => KeyF8,
            pancurses::Input::KeyF9 => KeyF9,
            pancurses::Input::KeyF10 => KeyF10,
            pancurses::Input::KeyF11 => KeyF11,
            pancurses::Input::KeyF12 => KeyF12,
            _ => KeyUnknown,
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}
