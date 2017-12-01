mod termion;

mod pancurses;

use std::io::{StdinLock, StdoutLock};

use state::GameState;

pub enum Type {
    Termion, Pancurses
}

pub trait IO {
    fn process_input(&mut self, game_state: &mut GameState);

    fn render_output(&mut self, game_state: &GameState);
}

pub fn create<'a>(io_type: Type, stdin: StdinLock<'a>,
                  stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match io_type {
        Type::Termion => {
            Box::new(termion::Terminal::new(stdin, stdout))
        },
        Type::Pancurses => {
            Box::new(pancurses::Terminal::new())
        }
    }
}
