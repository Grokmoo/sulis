mod pancurses;

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

pub mod mouse_event;
pub use self::mouse_event::MouseEvent;

mod input_action;
pub use self::input_action::InputAction;

use std::io::{StdinLock, StdoutLock};
use std::cell::{RefMut, Ref};

use state::GameState;
use config::{Config, IOAdapter};
use ui::WidgetBase;

pub trait IO {
    fn init(&mut self, config: &Config);

    fn process_input(&mut self, game_state: &mut GameState,
                     root: RefMut<WidgetBase>);

    fn render_output(&mut self, game_state: &GameState,
                     root: Ref<WidgetBase>);

    fn get_display_size(&self) -> (i32, i32);
}

pub trait TextRenderer {
    fn render_char(&mut self, c: char);

    fn render_string(&mut self, s: &str);

    fn set_cursor_pos(&mut self, x: i32, y: i32);
}

pub fn create<'a>(adapter: IOAdapter, _stdin: StdinLock<'a>,
                  _stdout: StdoutLock<'a>) -> Box<IO + 'a> {
    match adapter{
        IOAdapter::Pancurses => {
            Box::new(pancurses::Terminal::new())
        },
    }
}
