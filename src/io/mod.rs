mod termion;

mod pancurses;

use std::io::{StdinLock, StdoutLock};

use state::GameState;
use config::IOAdapter;

pub trait IO {
    fn process_input(&mut self, game_state: &mut GameState);

    fn render_output(&mut self, game_state: &GameState);
}

pub fn create<'a>(adapter: IOAdapter, stdin: StdinLock<'a>,
                  stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match adapter{
        IOAdapter::Termion => {
            Box::new(termion::Terminal::new(stdin, stdout))
        },
        IOAdapter::Pancurses => {
            Box::new(pancurses::Terminal::new())
        }
    }
}
