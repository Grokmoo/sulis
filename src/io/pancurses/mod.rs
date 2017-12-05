use pancurses::{self, Input};

use io::IO;

use state::GameState;

pub struct Terminal {
    window: pancurses::Window
}

impl Terminal {
    pub fn new() -> Terminal {
        let window = pancurses::initscr();
        pancurses::resize_term(40, 80);
        window.nodelay(true);
        window.keypad(true);
        pancurses::noecho();
        pancurses::curs_set(0);

        Terminal {
            window
        }
    }
}

impl IO for Terminal {
    fn process_input(&mut self, state: &mut GameState) {
        match self.window.getch() {
            Some(Input::Character(c)) => {
                state.handle_input(c);
            }
            Some(input) => {
                self.window.addstr(&format!("{:?}", input));
                println!("{:?}", input);
            }
            None => (),
        }
    }

    fn render_output(&mut self, state: &GameState) {
        self.window.erase();

        let ref area = state.area_state().area;
        self.window.printw(&format!("{}\n", area.name));

        for y in 0..area.height {
            for x in 0..area.width {
                self.window.printw(&format!("{}", state.area_state().get_display(x, y)));
            }
            self.window.printw("\n");
        }

    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}
