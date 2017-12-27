use std::rc::Rc;
use std::cell::RefCell;

use ui::{Cursor, Widget};
use io::event::Kind;
use io::{Event, KeyboardEvent};
use config::CONFIG;

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
    ClickCursor,
    ToggleInventory,
    ToggleCharacter,
    Exit,
}

impl InputAction {
    pub fn handle_keyboard_input(input: KeyboardEvent,
                                 root: Rc<RefCell<Widget>>) {

        debug!("Received {:?}", input);
        let action = {
            let action = CONFIG.get_input_action(input);

            match action {
                None => return,
                Some(action) => *action
            }
        };

        use io::InputAction::*;
        match action {
            MoveCursorUp => Cursor::move_by(root, 0, -1),
            MoveCursorDown => Cursor::move_by(root, 0, 1),
            MoveCursorLeft => Cursor::move_by(root, -1, 0),
            MoveCursorRight => Cursor::move_by(root, 1, 0),
            ClickCursor => Cursor::click(root),
            _ => InputAction::fire_action(action, root),
        }
    }

    fn fire_action(action: InputAction, root: Rc<RefCell<Widget>>) {
        debug!("Firing action {:?}", action);

        let event = Event::new(Kind::KeyPress(action), Cursor::get_x(), Cursor::get_y());
        Widget::dispatch_event(&root, event);
    }
}
