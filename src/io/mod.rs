mod pancurses;

mod keyboard_input;
pub use self::keyboard_input::KeyboardInput;

mod input_action;
pub use self::input_action::InputAction;

use std::io::{StdinLock, StdoutLock};

use state::GameState;
use config::{Config, IOAdapter};
use ui::WidgetState;

pub trait IO {
    fn init(&mut self, config: &Config);

    fn process_input(&mut self, game_state: &mut GameState, root: &mut WidgetState);

    fn render_output(&mut self, game_state: &GameState, root: &WidgetState);

    fn get_display_size(&self) -> (i32, i32);
}

pub trait TextRenderer {
    fn render_char(&mut self, c: char);

    fn render_string(&mut self, s: &str);

    fn set_cursor_pos(&mut self, x: u32, y: u32);
}

pub fn create<'a>(adapter: IOAdapter, _stdin: StdinLock<'a>,
                  _stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match adapter{
        IOAdapter::Pancurses => {
            Box::new(pancurses::Terminal::new())
        },
    }
}
