use state::GameState;
use ui::{Border, Window, Widget};
use io::event::Kind;
use io::Event;
use resource::ResourceSet;

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
                       root: &mut Widget) {
        use self::InputAction::*;

        debug!("Firing action {:?}", action);
        match action {
            MoveCursorUp => state.cursor_move_by(root, 0, -1),
            MoveCursorDown => state.cursor_move_by(root, 0, 1),
            MoveCursorLeft => state.cursor_move_by(root, -1, 0),
            MoveCursorRight => state.cursor_move_by(root, 1, 0),
            MoveToCursor => state.cursor_click(root),
            Exit => {
                // let mut window = Widget::with_position(Window::new("Really Quit?"),
                //     root.state.inner_size,
                //     root.state.inner_position);
                // window.state.set_background(ResourceSet::get_image("background"));
                // window.state.set_border(Border::as_uniform(1));
                // root.add_child(window);
                state.set_exit();
                true
            }
            _ => {
                let event = Event::new(Kind::KeyPress(action),
                                       state.cursor.x, state.cursor.y);
                root.dispatch_event(state, event)
            },
        };
    }
}
