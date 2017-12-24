use std::rc::Rc;
use std::cell::RefCell;

use state::GameState;
use ui::{Button, Label, Widget, WidgetKind};

pub struct Window { }

impl Window {
    pub fn new() -> Rc<Window> {
        Rc::new(Window {
        })
    }
}

impl WidgetKind for Window {
    fn get_name(&self) -> &str {
        "window"
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let label = Widget::with_theme(Label::empty(), "title");

        let cancel = Widget::with_theme(
            Button::with_callback(Rc::new(|_kind, widget| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            "cancel");

        let quit = Widget::with_theme(
            Button::with_callback(Rc::new(|_kind, _widget| {
                GameState::set_exit();
            })),
            "quit");

        vec![cancel, quit, label]
    }
}
