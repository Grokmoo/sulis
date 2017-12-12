use state::GameState;
use io::{InputAction, TextRenderer};
use io::event::ClickKind;
use resource::Point;

pub struct EmptyWidget { }
impl<'a> WidgetKind<'a> for EmptyWidget {
    fn get_name(&self) -> &str {
        "Empty"
    }
}

//// Trait for implementations of different Widgets.  This is held by a 'WidgetState'
//// object which contains the common functionality across all Widgets.
pub trait WidgetKind<'a> {
    fn draw_text_mode(&self, _renderer: &mut TextRenderer) { }

    fn get_name(&self) -> &str;

    fn on_mouse_click(&self, _state: &mut GameState,
                      _kind: ClickKind, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_move(&self, _state: &mut GameState,
                      _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_enter(&self, _state: &mut GameState, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_exit(&self, _state: &mut GameState, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_scroll(&self, _state: &mut GameState,
                         _scroll: i32, _mouse_pos: Point) -> bool {
        false
    }

    fn on_key_press(&self, _state: &mut GameState,
                    _key: InputAction, _mouse_pos: Point) -> bool {
        false
    }
}
