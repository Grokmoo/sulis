use pancurses;

use std::time::Instant;
use std::cell::{Ref, RefMut};

use io;
use io::keyboard_event::Key;
use io::{IO, KeyboardEvent, TextRenderer};

use state::GameState;
use ui::WidgetBase;
use config::Config;
use animation;

pub struct Terminal {
    window: pancurses::Window,
    start_time: Instant,
}

impl Terminal {
    pub fn new() -> Terminal {
        let window = pancurses::initscr();
        window.nodelay(true);
        window.keypad(true);
        pancurses::noecho();
        pancurses::curs_set(0);
        pancurses::nonl();

        Terminal {
            window,
            start_time: Instant::now(),
        }
    }

    #[cfg(target_os = "windows")]
    fn size_terminal(&mut self, config: &Config) {
        pancurses::resize_term(config.display.height as i32,
                               config.display.width as i32);
    }

    #[cfg(not(target_os = "windows"))]
    fn size_terminal(&mut self, _config: &Config) {
        // do nothing
    }
}

impl IO for Terminal {
    fn init(&mut self, config: &Config) {
        self.size_terminal(config);
    }

    fn process_input(&mut self, state: &mut GameState, root: RefMut<WidgetBase>) {
        let input = self.window.getch();
        if let None = input {
            return;
        }

        let input = match input.unwrap() {
            pancurses::Input::Character(c) => io::match_char(c),
            input => match_special(input),
        };
        let input = KeyboardEvent { key: input };

        state.handle_keyboard_input(input, root);
    }

    fn render_output(&mut self, state: &GameState, root: Ref<WidgetBase>) {
        self.window.erase();

        let millis = animation::get_elapsed_millis(self.start_time.elapsed());
        state.draw_text_mode(self as &mut Terminal, root, millis);
    }

    fn get_display_size(&self) -> (i32, i32) {
        (self.window.get_max_x(), self.window.get_max_y())
    }
}

impl TextRenderer for Terminal {
    fn render_char(&mut self, c: char) {
        self.window.addch(c);
    }

    fn render_string(&mut self, s: &str) {
        self.window.addstr(s);
    }

    fn set_cursor_pos(&mut self, x: i32, y: i32) {
        self.window.mv(y, x);
    }

    fn get_display_size(&self) -> (i32, i32) {
        (self.window.get_max_x(), self.window.get_max_y())
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
