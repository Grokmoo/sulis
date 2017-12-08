use std::io::{Bytes, Read, Write, StdinLock, StdoutLock};
use std::cell::{Ref, RefMut};
use std::time::Instant;

use config::Config;
use io::keyboard_event::Key;
use io::{self, KeyboardEvent, IO, TextRenderer};
use state::GameState;
use ui::WidgetBase;
use animation;

use termion::{self, async_stdin};
use termion::raw::{RawTerminal, IntoRawMode};
use termion::input::{TermRead, Keys};

pub struct Terminal<'a> {
    stdin: Bytes<termion::AsyncReader>,
    stdout: RawTerminal<StdoutLock<'a>>,
    start_time: Instant,
}

impl<'a> Terminal<'a> {
    pub fn new(stdin: StdinLock<'a>, stdout: StdoutLock<'a>) -> Terminal<'a> {
        let stdout = stdout.into_raw_mode().unwrap();
        //let stdin = stdin.keys();
        let stdin = async_stdin().bytes();

        Terminal {
            stdin,
            stdout,
            start_time: Instant::now(),
        }
    }
}

impl<'a> IO for Terminal<'a> {
    fn init(&mut self, _config: &Config) {
        write!(self.stdout, "{}", termion::cursor::Hide).unwrap();
    }

    fn process_input(&mut self, state: &mut GameState, root: RefMut<WidgetBase>) {
        let b = self.stdin.next();
        if let None = b { return; }
        let b = b.unwrap();
        if let Err(_) = b { return; }
        let b = b.unwrap();

        let input = io::match_char(b as char);
        let input = KeyboardEvent { key: input };

        state.handle_keyboard_input(input, root);
        // use termion::event::Key::*;
        // let input = match b {
        //     Char(c) => io::match_char(c),
        //     input => match_special(input),
        // };
        // let input = KeyboardEvent { key: input };
        //
        // state.handle_keyboard_input(input, root);
    }

    fn render_output(&mut self, state: &GameState, root: Ref<WidgetBase>) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();

        let millis = animation::get_elapsed_millis(self.start_time.elapsed());

        state.draw_text_mode(self as &mut Terminal, root, millis);
        self.stdout.flush().unwrap();
    }

    fn get_display_size(&self) -> (i32, i32) {
        let (w, h) = termion::terminal_size().unwrap();

        (w as i32, h as i32)
    }
}

impl<'a> TextRenderer for Terminal<'a> {
    fn render_char(&mut self, c: char) {
        write!(self.stdout, "{}", c).unwrap();
    }

    fn render_string(&mut self, s: &str) {
        write!(self.stdout, "{}", s).unwrap();
    }

    fn set_cursor_pos(&mut self, x: i32, y: i32) {
        write!(self.stdout, "{}",
               termion::cursor::Goto(x as u16 + 1, y as u16 + 1)).unwrap();
    }
}

fn match_special(input: termion::event::Key) -> Key {
    use termion::event::Key::*;
    use io::keyboard_event::Key::*;
    match input {
        Backspace => KeyBackspace,
        Left => KeyLeft,
        Right => KeyRight,
        Up => KeyUp,
        Down => KeyDown,
        Home => KeyHome,
        End => KeyEnd,
        PageUp => KeyPageUp,
        PageDown => KeyPageDown,
        Insert => KeyInsert,
        Esc => KeyEscape,

        F(1) => KeyF1,
        F(2) => KeyF2,
        F(3) => KeyF3,
        F(4) => KeyF4,
        F(5) => KeyF5,
        F(6) => KeyF6,
        F(7) => KeyF7,
        F(8) => KeyF8,
        F(9) => KeyF9,
        F(10) => KeyF10,
        F(11) => KeyF11,
        F(12) => KeyF12,

        _ => KeyUnknown,
    }
}
