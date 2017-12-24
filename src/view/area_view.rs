use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use state::{AreaState, GameState};
use ui::{WidgetKind, Widget};
use io::{InputAction, TextRenderer};
use io::event::ClickKind;
use resource::Point;

pub struct AreaView {
    area_state: Rc<RefCell<AreaState>>,
    mouse_over: Rc<RefCell<Widget>>,
}

impl AreaView {
    pub fn new(area_state: &Rc<RefCell<AreaState>>,
               mouse_over: Rc<RefCell<Widget>>) -> Rc<AreaView> {
        Rc::new(AreaView {
            area_state: Rc::clone(area_state),
            mouse_over: mouse_over,
        })
    }

}

impl WidgetKind for AreaView {
    fn get_name(&self) -> &str {
        "area"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let width = self.area_state.borrow().area.width;
        let height = self.area_state.borrow().area.height;
        widget.borrow_mut().state.set_max_scroll_pos(width, height);

        Vec::with_capacity(0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      widget: &Widget, _millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.inner_size;

        let state = self.area_state.borrow();
        let ref area = state.area;

        let max_x = cmp::min(s.width, area.width - widget.state.scroll_pos.x);
        let max_y = cmp::min(s.height, area.height - widget.state.scroll_pos.y);

        renderer.set_cursor_pos(0, 0);

        for y in 0..max_y {
            renderer.set_cursor_pos(p.x, p.y + y);
            for x in 0..max_x {
                renderer.render_char(state.get_display(x + widget.state.scroll_pos.x,
                                                       y + widget.state.scroll_pos.y));
            }
        }
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>,
                    key: InputAction, _mouse_pos: Point) -> bool {

        use io::InputAction::*;
        match key {
           ScrollUp => widget.borrow_mut().state.scroll(0, -1),
           ScrollDown => widget.borrow_mut().state.scroll(0, 1),
           ScrollLeft => widget.borrow_mut().state.scroll(-1, 0),
           ScrollRight => widget.borrow_mut().state.scroll(1, 0),
           _ => return false,
        };
        true
    }

    fn on_mouse_click(&self, widget: &Rc<RefCell<Widget>>,
                _kind: ClickKind, mouse_pos: Point) -> bool {
        let pc = GameState::pc();
        let size = pc.borrow().size();
        let pos = &widget.borrow().state.position;
        let x = (mouse_pos.x - pos.x) - size / 2;
        let y = (mouse_pos.y - pos.y) - size / 2;
        if x >= 0 && y >= 0 {
            GameState::pc_move_to(x + widget.borrow().state.scroll_pos.x, y +
                             widget.borrow().state.scroll_pos.y);
        }

        true
    }

    fn on_mouse_move(&self, widget: &Rc<RefCell<Widget>>,
                      mouse_pos: Point) -> bool {
        self.super_on_mouse_enter(widget);
        self.mouse_over.borrow_mut().state.set_text(&format!("[{},{}]",
            mouse_pos.x - 1, mouse_pos.y - 1));
        true
    }

    fn on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>,
                     _mouse_pos: Point) -> bool {
        self.super_on_mouse_exit(widget);
        self.mouse_over.borrow_mut().state.set_text("");
        true
    }
}
