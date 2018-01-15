use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::Widget;

pub struct ChangeListener<T> {
    cb: Box<Fn(&T)>,
    id: &'static str,
}

impl<T> ChangeListener<T> {
    pub fn new(id: &'static str, cb: Box<Fn(&T)>) -> ChangeListener<T> {
        ChangeListener {
            cb,
            id,
        }
    }

    pub fn invalidate(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().invalidate_children();
            }),
            id,
        }
    }

    pub fn call(&self, t: &T) {
        (self.cb)(t);
    }

    pub fn id(&self) -> &'static str {
        self.id
    }
}
