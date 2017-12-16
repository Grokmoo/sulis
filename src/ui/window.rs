use std::rc::Rc;
use std::cell::RefCell;

use ui::{Button, Label, Widget, WidgetKind};

pub struct Window { }

impl Window {
    pub fn new() -> Rc<Window> {
        Rc::new(Window {
        })
    }
}

impl<'a> WidgetKind<'a> for Window {
    fn get_name(&self) -> &str {
        "window"
    }

    fn layout(&self, widget: &mut Widget<'a>) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        let label = Widget::with_theme(Label::empty(), "title");

        let cancel = Widget::with_theme(
            Button::new(Box::new(|widget, _game_state| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            "cancel");

        let quit = Widget::with_theme(
            Button::new(Box::new(|_widget, game_state| { game_state.set_exit(); })),
            "quit");

        vec![cancel, quit, label]
    }
}
