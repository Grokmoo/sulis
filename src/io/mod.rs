mod termion;

use std::io::{StdinLock, StdoutLock};

use state::GameState;

pub enum Type {
    Termion,
}

pub trait IO {
    fn process_input(&mut self, game_state: &mut GameState);

    fn render_output(&mut self, game_state: &GameState);
}

pub fn create<'a>(io_type: Type, stdin: StdinLock<'a>,
                  stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match io_type {
        Type::Termion => {
            let term = termion::Terminal::new(stdin, stdout);
            Box::new(term)
        }
    }
}
