use std::rc::Rc;
use std::cell::RefCell;

use state::GameState;
use ui::{Border, Size, Window, Widget, EmptyWidget};
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
    pub fn fire_action<'a>(action: InputAction, game_state: &mut GameState<'a>,
                       root: Rc<RefCell<Widget<'a>>>) {
        use self::InputAction::*;

        debug!("Firing action {:?}", action);
        match action {
            MoveCursorUp => game_state.cursor_move_by(root, 0, -1),
            MoveCursorDown => game_state.cursor_move_by(root, 0, 1),
            MoveCursorLeft => game_state.cursor_move_by(root, -1, 0),
            MoveCursorRight => game_state.cursor_move_by(root, 1, 0),
            MoveToCursor => game_state.cursor_click(root),
            Exit => {
                if root.borrow().has_modal() {
                    false
                } else {
                    let exit_window = build_exit_window(&root);
                    Widget::add_children_to(&root, exit_window);
                    true
                }
            }
            _ => {
                let event = Event::new(Kind::KeyPress(action),
                                       game_state.cursor.x, game_state.cursor.y);
                Widget::dispatch_event(&root, game_state, event)
            },
        };
    }
}

fn build_exit_window<'a>(root: &Rc<RefCell<Widget<'a>>>) ->
    Vec<Rc<RefCell<Widget<'a>>>> {

    let ref state = root.borrow().state;
    let mut window = Widget::with_border(
        Window::new("Exit Game?",
                    Widget::with_defaults(EmptyWidget::new())),
        Size::new(26, 6),
        state.get_centered_position(20, 10),
        Border::as_uniform(1));
    window.borrow_mut().state.set_modal(true);
    Widget::set_background(&mut window, "background");

    vec![window]
}
