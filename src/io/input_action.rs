use std::cell::RefMut;

use state::GameState;
use ui::WidgetBase;
use io::event::Kind;
use io::Event;

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
    pub fn fire_action(action: InputAction, state: &mut GameState,
                       mut root: RefMut<WidgetBase>) {
        use self::InputAction::*;

        debug!("Firing action {:?}", action);
        match action {
            MoveCursorUp => state.cursor_move_by(root, 0, -1),
            MoveCursorDown => state.cursor_move_by(root, 0, 1),
            MoveCursorLeft => state.cursor_move_by(root, -1, 0),
            MoveCursorRight => state.cursor_move_by(root, 1, 0),
            MoveToCursor => state.cursor_click(root),
            Exit => state.set_exit(),
            _ => {
                let event = Event::new(Kind::KeyPress(action),
                                       state.cursor.x, state.cursor.y);
                root.dispatch_event(state, event)
            },
        };
    }
}
