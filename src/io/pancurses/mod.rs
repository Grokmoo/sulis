use pancurses;

use io::{IO, KeyboardInput, TextRenderer};
use std::time::Instant;

use state::GameState;
use ui::WidgetState;
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
}

impl IO for Terminal {
    fn init(&mut self, config: &Config) {
        pancurses::resize_term(config.display.height as i32,
                               config.display.width as i32);
    }

    fn process_input(&mut self, state: &mut GameState, root: &mut WidgetState) {
        let input = self.window.getch();
        if let None = input {
            return;
        }

        let input = match input.unwrap() {
            pancurses::Input::Character(c) => match_char(c),
            input => match_special(input),
        };

        state.handle_keyboard_input(input, root);
    }

    fn render_output(&mut self, state: &GameState, root: &WidgetState) {
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

    fn set_cursor_pos(&mut self, x: u32, y: u32) {
        self.window.mv(y as i32, x as i32);
    }
}

fn match_char(c: char) -> KeyboardInput {
    use io::KeyboardInput::*;
    match c {
        'a' | 'A' => KeyA,
        'b' | 'B' => KeyB,
        'c' | 'C' => KeyC,
        'd' | 'D' => KeyD,
        'e' | 'E' => KeyE,
        'f' | 'F' => KeyF,
        'g' | 'G' => KeyG,
        'h' | 'H' => KeyH,
        'i' | 'I' => KeyI,
        'j' | 'J' => KeyJ,
        'k' | 'K' => KeyK,
        'l' | 'L' => KeyL,
        'm' | 'M' => KeyM,
        'n' | 'N' => KeyN,
        'o' | 'O' => KeyO,
        'p' | 'P' => KeyP,
        'q' | 'Q' => KeyQ,
        'r' | 'R' => KeyR,
        's' | 'S' => KeyS,
        't' | 'T' => KeyT,
        'u' | 'U' => KeyU,
        'v' | 'V' => KeyV,
        'w' | 'W' => KeyW,
        'x' | 'X' => KeyX,
        'y' | 'Y' => KeyY,
        'z' | 'Z' => KeyZ,

        '0' | ')' => Key0,
        '1' | '!' => Key1,
        '2' | '@' => Key2,
        '3' | '#' => Key3,
        '4' | '$' => Key4,
        '5' | '%' => Key5,
        '6' | '^' => Key6,
        '7' | '&' => Key7,
        '8' | '*' => Key8,
        '9' | '(' => Key9,

        '\u{1b}' => KeyEscape,
        '\u{8}' => KeyBackspace,
        '\t' => KeyTab,
        ' ' => KeySpace,
        '\r' => KeyEnter,

        '~' | '`' => KeyGrave,
        '-' | '_' => KeyMinus,
        '=' | '+' => KeyEquals,
        '[' | '{' => KeyLeftBracket,
        ']' | '}' => KeyRightBracket,
        ';' | ':' => KeySemicolon,
        '\'' | '"' => KeySingleQuote,
        ',' | '<' => KeyComma,
        '.' | '>' => KeyPeriod,
        '/' | '?' => KeySlash,
        '\\' | '|' => KeyBackslash,


        _ => KeyUnknown,
    }
}

fn match_special(c: pancurses::Input) -> KeyboardInput {
    use io::KeyboardInput::*;
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
