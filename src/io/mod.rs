mod terminal;

use std::io::{StdinLock, StdoutLock};

use state::GameState;

pub enum Type {
    Terminal,
}

pub trait IO {
    fn process_input(&mut self, game_state: &mut GameState);

    fn render_output(&mut self, game_state: &GameState);
}

pub fn create<'a>(io_type: Type, stdin: StdinLock<'a>,
                  stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match io_type {
        Type::Terminal => {
            let term = terminal::Terminal::new(stdin, stdout);
            Box::new(term)
        }
    }
}
