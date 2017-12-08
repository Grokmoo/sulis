mod pancurses_adapter;
mod termion_adapter;

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

pub mod mouse_event;
pub use self::mouse_event::MouseEvent;

mod input_action;
pub use self::input_action::InputAction;

use std::cell::{RefMut, Ref};

use state::GameState;
use config::{Config, IOAdapter};
use ui::WidgetBase;
use io::keyboard_event::Key;

pub trait IO {
    fn init(&mut self, config: &Config);

    fn process_input(&mut self, game_state: &mut GameState,
                     root: RefMut<WidgetBase>);

    fn render_output(&mut self, game_state: &GameState,
                     root: Ref<WidgetBase>);

    fn get_display_size(&self) -> (i32, i32);
}

pub trait TextRenderer {
    fn render_char(&mut self, c: char);

    fn render_string(&mut self, s: &str);

    fn set_cursor_pos(&mut self, x: i32, y: i32);

    fn get_display_size(&self) -> (i32, i32);
}

pub fn create<'a>(adapter: IOAdapter) -> Box<IO + 'a> {
    match adapter{
        IOAdapter::Pancurses => {
            Box::new(pancurses_adapter::Terminal::new())
        },
        IOAdapter::Termion => {
            Box::new(termion_adapter::Terminal::new())
        },
    }
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
