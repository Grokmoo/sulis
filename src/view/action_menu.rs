use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Cursor, Label, Widget, WidgetKind};
use grt::io::event::ClickKind;
use grt::util::Point;

use state::AreaState;

pub struct ActionMenu {
    area_state: Rc<RefCell<AreaState>>,
    area_pos: Point,
}

impl ActionMenu {
    pub fn new(area_state: Rc<RefCell<AreaState>>, x: i32, y: i32) -> Rc<ActionMenu> {
        Rc::new(ActionMenu {
            area_state,
            area_pos: Point::new(x, y),
        })
   }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str {
        "action_menu"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");
        vec![title]
    }

    fn on_mouse_click(&self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) -> bool {
        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }

        true
    }
}
