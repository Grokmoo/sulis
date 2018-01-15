use std::rc::Rc;
use std::cell::RefCell;

use ui::{Button, Callback, Label, Widget, WidgetKind};

pub struct ConfirmationWindow {
    accept_callback: Callback,
}

impl ConfirmationWindow {
    pub fn new(accept_callback: Callback) -> Rc<ConfirmationWindow> {
        Rc::new(ConfirmationWindow {
            accept_callback
        })
    }
}

impl WidgetKind for ConfirmationWindow {
    fn get_name(&self) -> &str {
        "confirmation_window"
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let label = Widget::with_theme(Label::empty(), "title");

        let cancel = Widget::with_theme(Button::empty(), "cancel");
        cancel.borrow_mut().state.add_callback(Callback::remove_parent());

        let accept = Widget::with_theme(Button::empty(), "accept");
        accept.borrow_mut().state.add_callback(self.accept_callback.clone());

        vec![cancel, accept, label]
    }
}
