use std::io::{Bytes, Read, Write, stdout};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::time::Instant;
use std;

use config::Config;
use io::{self, KeyboardEvent, IO, TextRenderer};
use io::keyboard_event::Key;
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

    fn process_input(&mut self, state: &mut GameState, root: Rc<RefCell<WidgetBase>>) {
        let mut buf: Vec<u8> = Vec::new();

        loop {
            let b = self.stdin.next();
            if let None = b { break; }
            let b = b.unwrap();
            if let Err(_) = b { break; }
            buf.push(b.unwrap());
        }

        if buf.len() == 0 { return; }

        trace!("Processed {} bytes of input: {:?}", buf.len(), buf);

        while buf.len() > 0 {
            let b = buf.remove(0);

            let input = match b {
                27 => match_special(&mut buf), // escape character
                127 => io::keyboard_event::Key::KeyBackspace,
                _ => io::match_char(b as char),
            };

            let input = KeyboardEvent { key: input };

            state.handle_keyboard_input(input, root.borrow_mut());
        }
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

fn match_special(buf: &mut Vec<u8>) -> Key {
    use io::keyboard_event::Key::*;

    if buf.len() < 2 { return KeyUnknown; }

    match buf.remove(0) {
        91 => {
            return match buf.remove(0) {
                65 => KeyUp,
                66 => KeyDown,
                67 => KeyRight,
                68 => KeyLeft,

                49 => {
                    if buf.len() == 0 { return KeyUnknown; }

                    return match buf.remove(0) {
                        53 => remove_one_more(KeyF5, buf),
                        55 => remove_one_more(KeyF6, buf),
                        56 => remove_one_more(KeyF7, buf),
                        57 => remove_one_more(KeyF8, buf),
                        126 => KeyHome,
                        _ => KeyUnknown,
                    };
                },
                50 => {
                    if buf.len() == 0 { return KeyUnknown; }

                    return match buf.remove(0) {
                        48 => remove_one_more(KeyF9, buf),
                        49 => remove_one_more(KeyF10, buf),
                        51 => remove_one_more(KeyF11, buf),
                        52 => remove_one_more(KeyF12, buf),
                        126 => KeyInsert,
                        _ => KeyUnknown,
                    };
                },
                51 => remove_one_more(KeyDelete, buf),
                52 => remove_one_more(KeyEnd, buf),
                53 => remove_one_more(KeyPageUp, buf),
                54 => remove_one_more(KeyPageDown, buf),
                _ => KeyUnknown,
            };
        },
        79 => {
            return match buf.remove(0) {
                80 => KeyF1,
                81 => KeyF2,
                82 => KeyF3,
                83 => KeyF4,
                _ => KeyUnknown,
            };
        },
        _ => { return KeyUnknown; }
    }
}

fn remove_one_more(key: Key, buf: &mut Vec<u8>) -> Key {
    if buf.len() != 0 {
        buf.remove(0);
    }

    key
}
