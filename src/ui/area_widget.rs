use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

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
    fn get_name(&self) -> &str {
        "Area"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, owner: &WidgetBase) {
        let scroll_x = 0;
        let scroll_y = 0;

        let p = owner.inner_position();
        let s = owner.inner_size();

        let state = self.area_state.borrow();
        let ref area = state.area;

        let max_x = cmp::min(s.width, area.width - scroll_x);
        let max_y = cmp::min(s.height, area.height - scroll_y);

        renderer.set_cursor_pos(0, 0);

        for y in 0..max_y {
            renderer.set_cursor_pos(p.x, p.y + y);
            for x in 0..max_x {
                renderer.render_char(state.get_display(x + scroll_x, y + scroll_y));
            }
        }
    }

    fn on_left_click(&self, parent: &mut WidgetBase, state: &mut GameState,
                x: i32, y: i32) -> bool {
        let size = state.pc().size() as i32;
        let x = (x - parent.position.x as i32) - size / 2;
        let y = (y - parent.position.y as i32) - size / 2;
        if x >= 0 && y >= 0 {
            state.pc_move_to(x, y);
        }

        true // consume the event
    }

    fn on_mouse_moved(&self, _parent: &mut WidgetBase, _state: &mut GameState,
                      x: i32, y: i32) -> bool {
        self.mouse_over.top_mut().set_text(&format!("[{},{}]", x, y));
        true
    }

    fn on_mouse_exited(&self, parent: &mut WidgetBase, _state: &mut GameState,
                       _x: i32, _y: i32) -> bool {
       parent.set_mouse_inside(false);
       self.mouse_over.top_mut().set_text("");
       true
    }
}
