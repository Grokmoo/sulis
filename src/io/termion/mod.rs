use std::io::{Write, StdinLock, StdoutLock};
use io::IO;
use state::GameState;

use termion;
use termion::raw::{RawTerminal, IntoRawMode};
use termion::input::{TermRead, Keys};

pub struct Terminal<'a> {
    stdin: Keys<StdinLock<'a>>,
    stdout: RawTerminal<StdoutLock<'a>>,
}

impl<'a> Terminal<'a> {
    pub fn new(stdin: StdinLock<'a>, stdout: StdoutLock<'a>) -> Terminal<'a> {
        let _guard = termion::init();
        
        let stdout = stdout.into_raw_mode().unwrap();
        let stdin = stdin.keys();

        Terminal {
            stdin,
            stdout,
        }
    }
}

impl<'a> IO for Terminal<'a> {
    fn process_input(&mut self, state: &mut GameState) {
        let b = self.stdin.next().unwrap().unwrap();
        use termion::event::Key::*;
        match b {
            Char(c) => state.handle_input(c),
            _ => (),
        };
    }

    fn render_output(&mut self, state: &GameState) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();

        let ref area = state.area_state().area;

        write!(self.stdout, "{}\n", area.name).unwrap();
        for y in 0..area.height {
            for x in 0..area.width {
                write!(self.stdout, "{}", state.area_state().get_display(x, y)).unwrap();
            }
            write!(self.stdout, "\n").unwrap();
        }

        self.stdout.flush().unwrap();
    }
}

