use std::rc::Rc;
use std::cell::RefCell;

use state::{AreaState, GameState};
use ui::{Widget, WidgetBase, Label, WidgetRef};
use io::TextRenderer;

pub struct AreaWidget<'a> {
    area_state: Rc<RefCell<AreaState<'a>>>,
    mouse_over: WidgetRef<'a, Label>,
}

impl<'a> AreaWidget<'a> {
    pub fn new(area_state: Rc<RefCell<AreaState<'a>>>,
               mouse_over: WidgetRef<'a, Label>) -> Rc<RefCell<AreaWidget<'a>>> {
        Rc::new(RefCell::new(AreaWidget {
            area_state,
            mouse_over,
        }))
    }
}

impl<'a> Widget for AreaWidget<'a> {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, owner: &WidgetBase) {
        let x_start = owner.x;
        let y_start = owner.y;
        let state = self.area_state.borrow();
        let ref area = state.area;
        for y in 0..area.height {
            renderer.set_cursor_pos(x_start, (y as i32) + y_start);
            for x in 0..area.width {
                renderer.render_char(state.get_display(x, y));
            }
        }
    }

    fn on_left_click(&self, parent: &WidgetBase, state: &mut GameState,
                x: i32, y: i32) -> bool {
        let size = state.pc().size() as i32;
        let x = (x - parent.x) - size / 2;
        let y = (y - parent.y) - size / 2;
        if x >= 0 && y >= 0 {
            state.pc_move_to(x as usize, y as usize);
        }

        true // consume the event
    }

    fn on_mouse_moved(&self, _parent: &WidgetBase, _state: &mut GameState,
                      x: i32, y: i32) -> bool {
        self.mouse_over.top_mut().set_text(&format!("[{},{}]", x, y));
        self.mouse_over.base_mut().set_position_centered(x, y + 2);
        true
    }
}
