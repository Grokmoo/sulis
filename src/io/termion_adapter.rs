use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::io::{Bytes, Read, Write, stdout};
use std;

use config::Config;
use io::{self, KeyboardEvent, IO};
use io::keyboard_event::Key;
use io::buffered_text_renderer::BufferedTextRenderer;
use state::GameState;
use ui::{Widget, Size};

use termion::screen::*;
use termion::{self, async_stdin};
use termion::raw::{RawTerminal, IntoRawMode};

pub struct Terminal {
    stdin: Bytes<termion::AsyncReader>,
    stdout: AlternateScreen<RawTerminal<std::io::Stdout>>,
    renderer: BufferedTextRenderer,
    size: Size,
}

impl Terminal {
    pub fn new(config: &Config) -> Terminal {
        debug!("Initialize Termion display adapter.");
        let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        let stdin = async_stdin().bytes();
        let size = Size::new(config.display.width, config.display.height);

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        Terminal {
            stdin,
            stdout,
            renderer: BufferedTextRenderer::new(size),
            size: size,
        }
    }
}

impl<'a> IO<'a> for Terminal {
    fn process_input(&mut self, state: &mut GameState<'a>,
                     root: Rc<RefCell<Widget<'a>>>) {
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

            state.handle_keyboard_input(input, Rc::clone(&root));
        }
    }

    fn render_output(&mut self, state: &GameState<'a>, root: Ref<Widget<'a>>,
                     millis: u32) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        self.renderer.clear();

        state.draw_text_mode(&mut self.renderer, root, millis);

        for y in 0..self.size.height {
            write!(self.stdout, "{}",
                   termion::cursor::Goto(1, y as u16 + 1)).unwrap();

            write!(self.stdout, "{}", self.renderer.get_line(y)).unwrap();
        }

        self.stdout.flush().unwrap();
    }
}

fn match_special(buf: &mut Vec<u8>) -> Key {
    use io::keyboard_event::Key::*;

    if buf.len() == 0 { return KeyEscape; }

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
