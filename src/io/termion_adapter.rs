use std::io::{Bytes, Read, Write, stdout};
use std::cell::{Ref, RefMut};
use std::time::Instant;
use std;

use config::Config;
use io::{self, KeyboardEvent, IO, TextRenderer};
use state::GameState;
use ui::WidgetBase;
use animation;

use termion::screen::*;
use termion::{self, async_stdin};
use termion::raw::{RawTerminal, IntoRawMode};

pub struct Terminal {
    stdin: Bytes<termion::AsyncReader>,
    stdout: AlternateScreen<RawTerminal<std::io::Stdout>>,
    start_time: Instant,

    write_buffer: String,
}

impl Terminal {
    pub fn new() -> Terminal {
        debug!("Initialize Termion display adapter.");
        let stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        //let stdout = stdout.into_raw_mode().unwrap();
        //let stdin = stdin.keys();
        let stdin = async_stdin().bytes();

        Terminal {
            stdin,
            stdout,
            start_time: Instant::now(),
            write_buffer: String::new(),
        }
    }

    fn write_buffered_text(&mut self) {
        if self.write_buffer.len() > 0 {
            write!(self.stdout, "{}", &self.write_buffer).unwrap();
            self.write_buffer.clear();
        }
    }
}

impl IO for Terminal {
    fn init(&mut self, _config: &Config) {
        trace!("Called init on termion adapter");
        write!(self.stdout, "{}", termion::cursor::Hide).unwrap();
    }

    fn process_input(&mut self, state: &mut GameState, root: RefMut<WidgetBase>) {
        // TODO handle reading multi byte special characters
        let b = self.stdin.next();
        if let None = b { return; }
        let b = b.unwrap();
        if let Err(_) = b { return; }
        let b = b.unwrap();

        let input = io::match_char(b as char);
        let input = KeyboardEvent { key: input };

        state.handle_keyboard_input(input, root);
    }

    fn render_output(&mut self, state: &GameState, root: Ref<WidgetBase>) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();

        let millis = animation::get_elapsed_millis(self.start_time.elapsed());

        state.draw_text_mode(self as &mut Terminal, root, millis);

        self.write_buffered_text();
        self.stdout.flush().unwrap();
    }

    fn get_display_size(&self) -> (i32, i32) {
        let (w, h) = termion::terminal_size().unwrap();

        (w as i32, h as i32)
    }
}

impl TextRenderer for Terminal {
    fn render_char(&mut self, c: char) {
        self.write_buffer.push(c);
        //write!(self.stdout, "{}", c).unwrap();
    }

    fn render_string(&mut self, s: &str) {
        self.write_buffer.push_str(s);
        //write!(self.stdout, "{}", s).unwrap();
    }

    fn set_cursor_pos(&mut self, x: i32, y: i32) {
        self.write_buffered_text();

        write!(self.stdout, "{}",
               termion::cursor::Goto(x as u16 + 1, y as u16 + 1)).unwrap();
    }

    fn get_display_size(&self) -> (i32, i32) {
        let (w, h) = termion::terminal_size().unwrap();

        (w as i32, h as i32)
    }
}

// fn match_special(input: termion::event::Key) -> Key {
//     use termion::event::Key::*;
//     use io::keyboard_event::Key::*;
//     match input {
//         Backspace => KeyBackspace,
//         Left => KeyLeft,
//         Right => KeyRight,
//         Up => KeyUp,
//         Down => KeyDown,
//         Home => KeyHome,
//         End => KeyEnd,
//         PageUp => KeyPageUp,
//         PageDown => KeyPageDown,
//         Insert => KeyInsert,
//         Esc => KeyEscape,
//
//         F(1) => KeyF1,
//         F(2) => KeyF2,
//         F(3) => KeyF3,
//         F(4) => KeyF4,
//         F(5) => KeyF5,
//         F(6) => KeyF6,
//         F(7) => KeyF7,
//         F(8) => KeyF8,
//         F(9) => KeyF9,
//         F(10) => KeyF10,
//         F(11) => KeyF11,
//         F(12) => KeyF12,
//
//         _ => KeyUnknown,
//     }
// }
