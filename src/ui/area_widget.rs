use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use state::{AreaState, GameState};
use ui::{BaseRef, Widget, WidgetBase, Label, WidgetRef};
use io::{InputAction, TextRenderer};
use io::event::ClickKind;
use resource::Point;

pub struct AreaWidget<'a> {
    area_state: Rc<RefCell<AreaState<'a>>>,
    mouse_over: WidgetRef<'a, Label<'a>>,

    scroll_x: i32,
    scroll_y: i32,
    base_ref: BaseRef<'a>,
    parent_width: i32,
    parent_height: i32,
}

impl<'a> AreaWidget<'a> {
    pub fn new(area_state: Rc<RefCell<AreaState<'a>>>,
               mouse_over: WidgetRef<'a, Label<'a>>) -> Rc<RefCell<AreaWidget<'a>>> {
        Rc::new(RefCell::new(AreaWidget {
            area_state,
            mouse_over,
            scroll_x: 0,
            scroll_y: 0,
            base_ref: BaseRef::new(),
            parent_width: 0,
            parent_height: 0,
        }))
    }

    pub fn scroll(&mut self, x: i32, y: i32) -> bool {
        let new_x = self.scroll_x + x;
        let new_y = self.scroll_y + y;

        if new_x < 0 || new_y < 0 { return false; }

        let width = self.area_state.borrow().area.width;
        let height = self.area_state.borrow().area.height;

        if new_x >= width - self.parent_width + 1 ||
            new_y >= height - self.parent_height + 1 {
            return false;
        }

        self.scroll_x = new_x;
        self.scroll_y = new_y;

        true
    }
}

impl<'a> Widget<'a> for AreaWidget<'a> {
    fn get_name(&self) -> &str {
        "Area"
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<WidgetBase<'a>>>) {
        self.base_ref.set_base(parent);
        self.parent_width = parent.borrow().size.width;
        self.parent_height = parent.borrow().size.height;
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        let p = self.base_ref.base().inner_position();
        let s = self.base_ref.base().inner_size();

        let state = self.area_state.borrow();
        let ref area = state.area;

        let max_x = cmp::min(s.width, area.width - self.scroll_x);
        let max_y = cmp::min(s.height, area.height - self.scroll_y);

        renderer.set_cursor_pos(0, 0);

        for y in 0..max_y {
            renderer.set_cursor_pos(p.x, p.y + y);
            for x in 0..max_x {
                renderer.render_char(state.get_display(x + self.scroll_x,
                                                       y + self.scroll_y));
            }
        }
    }

    fn on_key_press(&mut self, _state: &mut GameState,
                    key: InputAction, _mouse_pos: Point) -> bool {

        use io::InputAction::*;
        match key {
           ScrollUp => self.scroll(0, -1),
           ScrollDown => self.scroll(0, 1),
           ScrollLeft => self.scroll(-1, 0),
           ScrollRight => self.scroll(1, 0),
           _ => false,
        };
        true
    }

    fn on_mouse_click(&mut self, state: &mut GameState,
                _kind: ClickKind, mouse_pos: Point) -> bool {
        let size = state.pc().size();
        let pos = self.base_ref.base().position;
        let x = (mouse_pos.x - pos.x) - size / 2;
        let y = (mouse_pos.y - pos.y) - size / 2;
        if x >= 0 && y >= 0 {
            state.pc_move_to(x + self.scroll_x, y + self.scroll_y);
        }

        true // consume the event
    }

    fn on_mouse_move(&mut self, _state: &mut GameState,
                      mouse_pos: Point) -> bool {
        self.mouse_over.top_mut().set_text(&format!("[{},{}]",
                                                    mouse_pos.x, mouse_pos.y));
        true
    }

    fn on_mouse_exit(&mut self, _state: &mut GameState,
                     _mouse_pos: Point) -> bool {
        self.base_ref.base_mut().set_mouse_inside(false);
        self.mouse_over.top_mut().set_text("");
        true
    }
}
