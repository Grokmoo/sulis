use ui::WidgetBase;
use state::GameState;
use io::TextRenderer;

pub struct EmptyWidget { }
impl Widget for EmptyWidget { }

//// Trait for implementations of different Widgets.  This is held by a 'WidgetBase'
//// object which contains the common functionality across all Widgets.
pub trait Widget {
    fn draw_text_mode(&self, _renderer: &mut TextRenderer, _owner: &WidgetBase) { }

    fn on_left_click(&self, _parent: &WidgetBase, _state: &mut GameState,
                _x: i32, _y: i32) -> bool {
        false
    }

    fn on_middle_click(&self, _parent: &WidgetBase, _state: &mut GameState,
                _x: i32, _y: i32) -> bool {
        false
    }

    fn on_right_click(&self, _parent: &WidgetBase, _state: &mut GameState,
                _x: i32, _y: i32) -> bool {
        false
    }

    fn on_mouse_moved(&self, _parent: &WidgetBase, _state: &mut GameState,
                      _x: i32, _y: i32) -> bool {
        false
    }
}
