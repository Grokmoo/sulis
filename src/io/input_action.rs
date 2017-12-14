use std::rc::Rc;
use std::cell::RefCell;

use state::GameState;
use ui::{Border, Size, Window, Widget};
use io::event::Kind;
use io::Event;
use resource::Point;

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum InputAction {
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveToCursor,
    Exit,
}

impl InputAction {
    pub fn fire_action<'a>(action: InputAction, game_state: &mut GameState<'a>,
                       root: Rc<RefCell<Widget<'a>>>) {
        use self::InputAction::*;
        // use ui::widget::state;

        debug!("Firing action {:?}", action);
        match action {
            MoveCursorUp => game_state.cursor_move_by(root, 0, -1),
            MoveCursorDown => game_state.cursor_move_by(root, 0, 1),
            MoveCursorLeft => game_state.cursor_move_by(root, -1, 0),
            MoveCursorRight => game_state.cursor_move_by(root, 1, 0),
            MoveToCursor => game_state.cursor_click(root),
            Exit => {
                let mut window = Widget::with_border(
                    Window::new("Really Quit?"),
                    Size::new(20, 10),
                    Point::new(1, 1),
                    // Point::new(state(&root).inner_left() +
                    //            (state(&root).size.width - 20) / 2,
                    //            state(&root).inner_top() +
                    //            (state(&root).size.height - 10) / 2),
                    Border::as_uniform(1));
                window.borrow_mut().state.set_modal(true);
                Widget::set_background(&mut window, "background");
                Widget::add_child_to(&root, window);
                true
            }
            _ => {
                let event = Event::new(Kind::KeyPress(action),
                                       game_state.cursor.x, game_state.cursor.y);
                Widget::dispatch_event(&root, game_state, event)
            },
        };
    }
}
