mod pancurses_adapter;

#[cfg(not(windows))] mod termion_adapter;
#[cfg(windows)] use std::io::ErrorKind;

mod buffered_text_renderer;

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

pub mod event;
pub use self::event::Event;

mod input_action;
pub use self::input_action::InputAction;

use std::io::Error;

use state::GameState;
use config::{Config, IOAdapter};
use ui::Widget;
use io::keyboard_event::Key;

pub trait IO {
    fn process_input(&mut self, game_state: &mut GameState,
                     root: &mut Widget);

    fn render_output(&mut self, game_state: &GameState,
                     root: &Widget, millis: u32);
}

pub trait TextRenderer {
    fn render_char(&mut self, c: char);

    fn render_chars(&mut self, cs: &[char]);

    fn render_string(&mut self, s: &str);

    fn set_cursor_pos(&mut self, x: i32, y: i32);
}

pub fn create<'a>(config: &Config) -> Result<Box<IO + 'a>, Error> {
    match config.display.adapter {
        IOAdapter::Pancurses => {
            Ok(Box::new(pancurses_adapter::Terminal::new(config)))
        },
        IOAdapter::Termion => {
            get_termion_adapter(config)
        },
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_termion_adapter<'a>(config: &Config) -> Result<Box<IO + 'a>, Error> {
    Ok(Box::new(termion_adapter::Terminal::new(config)))
}

#[cfg(target_os = "windows")]
pub fn get_termion_adapter<'a>(_config: &Config) -> Result<Box<IO + 'a>, Error> {
    Err(Error::new(ErrorKind::InvalidInput,
                   "Termion display adapter is not supported on windows."))
}

pub(in::io) fn match_char(c: char) -> Key {
    use io::keyboard_event::Key::*;
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
