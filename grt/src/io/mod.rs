#[cfg(windows)] mod pancurses_adapter;
#[cfg(not(windows))] mod termion_adapter;
mod glium_adapter;

mod buffered_text_renderer;

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

pub mod event;
pub use self::event::Event;

mod input_action;
pub use self::input_action::InputAction;

use std::io::{ErrorKind, Error};
use std::rc::Rc;
use std::cell::{RefCell, Ref};

use config::{CONFIG, IOAdapter};
use ui::Widget;
use io::keyboard_event::Key;
use resource::Sprite;

pub trait IO {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>);

    fn render_output(&mut self, root: Ref<Widget>, millis: u32);
}

pub trait TextRenderer {
    fn render_char(&mut self, c: char);

    fn render_chars(&mut self, cs: &[char]);

    fn render_string(&mut self, s: &str);

    fn set_cursor_pos(&mut self, x: i32, y: i32);
}

#[derive(Debug, Clone)]
pub enum DrawListKind {
    Font,
    Sprite,
}

#[derive(Debug, Clone)]
pub struct DrawList {
    pub quads: Vec<[Vertex; 4]>,
    pub texture: String,
    pub kind: DrawListKind,
}

impl DrawList {
    /// Creates an empty DrawList.  Attempting to draw an empty DrawList will most
    /// likely result in a panic, you must use `append` to add vertices to this list
    /// if you intend to draw it.
    pub fn empty_sprite() -> DrawList {
        DrawList {
            quads: Vec::new(),
            texture: String::new(),
            kind: DrawListKind::Sprite,
        }
    }

    pub fn from_font(texture: &str, quads: Vec<[Vertex; 4]>) -> DrawList {
        DrawList {
            texture: texture.to_string(),
            quads,
            kind: DrawListKind::Font,
        }
    }

    pub fn from_sprite(sprite: &Rc<Sprite>, x: i32, y: i32, w: i32, h: i32) -> DrawList {
        let x_min = x as f32;
        let y_max = (CONFIG.display.height - y) as f32;
        let x_max = x_min + w as f32;
        let y_min = y_max - h as f32;
        let tc = &sprite.tex_coords;

        let quads = vec![[
            Vertex { position: [ x_min, y_max ], tex_coords: [tc[0], tc[1]] },
            Vertex { position: [ x_min, y_min ], tex_coords: [tc[2], tc[3]] },
            Vertex { position: [ x_max, y_max ], tex_coords: [tc[4], tc[5]] },
            Vertex { position: [ x_max, y_min ], tex_coords: [tc[6], tc[7]] },
        ]];

        DrawList {
            texture: sprite.id.to_string(),
            quads,
            kind: DrawListKind::Sprite,
        }
    }

    /// appends the contents of the other drawlist to this one, moving
    /// the vertex data out into this DrawList's vertex data.
    pub fn append(&mut self, other: &mut DrawList) {
        self.quads.append(&mut other.quads);
        if self.texture.is_empty() {
            self.texture = other.texture.to_string();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub fn create() -> Result<Box<IO>, Error> {
    match CONFIG.display.adapter {
        IOAdapter::Pancurses => get_pancurses_adapter(),
        IOAdapter::Termion => get_termion_adapter(),
        IOAdapter::Auto => get_auto_adapter(),
        IOAdapter::Glium => get_glium_adapter(),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_auto_adapter() -> Result<Box<IO>, Error> {
    get_termion_adapter()
}

pub fn get_glium_adapter() -> Result<Box<IO>, Error> {
    Ok(Box::new(glium_adapter::GliumDisplay::new()))
}

#[cfg(not(target_os = "windows"))]
pub fn get_termion_adapter() -> Result<Box<IO>, Error> {
    Ok(Box::new(termion_adapter::Terminal::new()))
}

#[cfg(not(target_os = "windows"))]
pub fn get_pancurses_adapter() -> Result<Box<IO>, Error> {
    Err(Error::new(ErrorKind::InvalidInput,
                   "Pancurses display adapter is only supported on windows.  Try 'Termion'"))
}

#[cfg(target_os = "windows")]
pub fn get_auto_adapter() -> Result<Box<IO>, Error> {
    get_pancurses_adapter()
}

#[cfg(target_os = "windows")]
pub fn get_termion_adapter() -> Result<Box<IO>, Error> {
    Err(Error::new(ErrorKind::InvalidInput,
                   "Termion display adapter is not supported on windows.  Try 'Pancurses'"))
}

#[cfg(target_os = "windows")]
pub fn get_pancurses_adapter() -> Result<Box<IO>, Error> {
    Ok(Box::new(pancurses_adapter::Terminal::new()))
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
