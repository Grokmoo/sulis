use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Button, Callback, Cursor, Label, list_box, ListBox, Widget, WidgetKind};
use grt::io::event::ClickKind;
use grt::util::Point;

use state::{AreaState, GameState};

pub struct ActionMenu {
    _area_state: Rc<RefCell<AreaState>>,
    area_pos: Point,
}

impl ActionMenu {
    pub fn new(area_state: Rc<RefCell<AreaState>>, x: i32, y: i32) -> Rc<ActionMenu> {
        Rc::new(ActionMenu {
            _area_state: area_state,
            area_pos: Point::new(x, y),
        })
    }

    pub fn move_callback(&self) -> Box<Fn()> {
        let pc = GameState::pc();
        let size = pc.borrow().size();
        let x = self.area_pos.x - size / 2;
        let y = self.area_pos.y - size / 2;
        Box::new(move || {
            GameState::pc_move_to(x, y);
        })
    }

    fn callback_with_removal(f: Box<Fn()>) -> Option<Callback<Button>> {
        Some(Callback::new(Rc::new(move |_kind, widget| {
            f();
            Widget::mark_removal_up_tree(&widget, 2);
        })))
    }

    pub fn fire_default_callback(&self) {
        (self.move_callback())();
    }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str {
        "action_menu"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry> = Vec::new();
        entries.push(list_box::Entry::new(
                "Move",
                ActionMenu::callback_with_removal(self.move_callback())
                ));

        let actions = Widget::with_theme(ListBox::new(entries), "actions");

        vec![title, actions]
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }

        true
    }
}
