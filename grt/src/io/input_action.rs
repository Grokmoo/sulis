use std::rc::Rc;
use std::cell::RefCell;

use ui::{Cursor, Widget};
use io::event::{ClickKind, Kind};
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
    ClickCursor,
    RightClickCursor,
    ToggleInventory,
    ToggleCharacter,
    Exit,
    MouseMove(i32, i32),
    MouseDown(ClickKind),
    MouseUp(ClickKind),
}

impl InputAction {
    pub fn handle_action(action: Option<InputAction>, root: Rc<RefCell<Widget>>) {
        let action = match action {
            None => return,
            Some(action) => action
        };

        debug!("Received action {:?}", action);

        Widget::remove_mouse_over(&root);
        use io::InputAction::*;
        match action {
            MoveCursorUp => Cursor::move_by(&root, 0, -1),
            MoveCursorDown => Cursor::move_by(&root, 0, 1),
            MoveCursorLeft => Cursor::move_by(&root, -1, 0),
            MoveCursorRight => Cursor::move_by(&root, 1, 0),
            ClickCursor => {
                Cursor::press(&root, ClickKind::Left);
                Cursor::release(&root, ClickKind::Left)
            }
            RightClickCursor => {
                Cursor::press(&root, ClickKind::Right);
                Cursor::release(&root, ClickKind::Right)
            },
            MouseMove(x, y) => Cursor::move_to(&root, x, y),
            MouseDown(kind) => Cursor::press(&root, kind),
            MouseUp(kind) => Cursor::release(&root, kind),
            _ => InputAction::fire_action(action, root),
        }
    }

    fn fire_action(action: InputAction, root: Rc<RefCell<Widget>>) {
        debug!("Firing action {:?}", action);

        let event = Event::new(Kind::KeyPress(action));
        Widget::dispatch_event(&root, event);
    }
}
