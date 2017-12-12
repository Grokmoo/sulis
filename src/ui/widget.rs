use std::rc::Rc;
use std::cell::RefCell;

use ui::WidgetBase;
use state::GameState;
use io::{InputAction, TextRenderer};
use io::event::ClickKind;
use resource::Point;

pub struct EmptyWidget { }
impl<'a> Widget<'a> for EmptyWidget {
    fn get_name(&self) -> &str {
        "Empty"
    }
}

//// Trait for implementations of different Widgets.  This is held by a 'WidgetBase'
//// object which contains the common functionality across all Widgets.
pub trait Widget<'a> {
    fn draw_text_mode(&self, _renderer: &mut TextRenderer) { }

    fn get_name(&self) -> &str;

    fn set_parent(&mut self, _parent: &Rc<RefCell<WidgetBase<'a>>>) { }

    fn on_mouse_click(&mut self, _state: &mut GameState,
                      _kind: ClickKind, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_move(&mut self, _state: &mut GameState,
                      _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_enter(&mut self, _state: &mut GameState, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_exit(&mut self, _state: &mut GameState, _mouse_pos: Point) -> bool {
        false
    }

    fn on_mouse_scroll(&mut self, _state: &mut GameState,
                         _scroll: i32, _mouse_pos: Point) -> bool {
        false
    }

    fn on_key_press(&mut self, _state: &mut GameState,
                    _key: InputAction, _mouse_pos: Point) -> bool {
        false
    }
}
