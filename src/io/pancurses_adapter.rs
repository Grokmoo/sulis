use std::rc::Rc;
use std::cell::{RefCell, Ref};

use pancurses;

use io::buffered_text_renderer::BufferedTextRenderer;
use io;
use io::keyboard_event::Key;
use io::{IO, KeyboardEvent};
use state::GameState;
use ui::{Widget, Size};
use config::Config;

pub struct Terminal {
    window: pancurses::Window,
    renderer: BufferedTextRenderer,
    size: Size,
}

impl Terminal {
    pub fn new(config: &Config) -> Terminal {
        debug!("Initialize Pancurses display adapter.");
        let window = pancurses::initscr();
        window.nodelay(true);
        window.keypad(true);
        pancurses::noecho();
        pancurses::curs_set(0);
        pancurses::nonl();

        Terminal::size_terminal(config);

        let size = Size::new(config.display.width, config.display.height);

        Terminal {
            window,
            size,
            renderer: BufferedTextRenderer::new(size),
        }
    }

    #[cfg(target_os = "windows")]
    fn size_terminal(config: &Config) {
        pancurses::resize_term(config.display.height as i32,
                               config.display.width as i32);
    }

    #[cfg(not(target_os = "windows"))]
    fn size_terminal(_config: &Config) {
        // do nothing
    }
}

impl IO for Terminal {
    fn process_input(&mut self, state: &mut GameState,
                     root: Rc<RefCell<Widget>>) {
        let input = self.window.getch();
        if let None = input {
            return;
        }

        let input = match input.unwrap() {
            pancurses::Input::Character(c) => io::match_char(c),
            input => match_special(input),
        };
        let input = KeyboardEvent { key: input };

        state.handle_keyboard_input(input, root);
    }

    fn render_output(&mut self, state: &GameState, root: Ref<Widget>,
                     millis: u32) {
        self.window.erase();
        self.renderer.clear();

        state.draw_text_mode(&mut self.renderer, root, millis);

        for y in 0..self.size.height {
            self.window.mv(y, 0);
            self.window.addstr(&self.renderer.get_line(y));
        }
    }
}

fn match_special(c: pancurses::Input) -> Key {
    use io::keyboard_event::Key::*;
    match c {
            pancurses::Input::KeyHome => KeyHome,
            pancurses::Input::KeyEnd => KeyEnd,
            pancurses::Input::KeyIC => KeyInsert,
            pancurses::Input::KeyDC => KeyDelete,
            pancurses::Input::KeyPPage => KeyPageUp,
            pancurses::Input::KeyNPage => KeyPageDown,
            pancurses::Input::KeyUp => KeyUp,
            pancurses::Input::KeyDown => KeyDown,
            pancurses::Input::KeyRight => KeyRight,
            pancurses::Input::KeyLeft => KeyLeft,
            pancurses::Input::KeyF1 => KeyF1,
            pancurses::Input::KeyF2 => KeyF2,
            pancurses::Input::KeyF3 => KeyF3,
            pancurses::Input::KeyF4 => KeyF4,
            pancurses::Input::KeyF5 => KeyF5,
            pancurses::Input::KeyF6 => KeyF6,
            pancurses::Input::KeyF7 => KeyF7,
            pancurses::Input::KeyF8 => KeyF8,
            pancurses::Input::KeyF9 => KeyF9,
            pancurses::Input::KeyF10 => KeyF10,
            pancurses::Input::KeyF11 => KeyF11,
            pancurses::Input::KeyF12 => KeyF12,
            _ => KeyUnknown,
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}
