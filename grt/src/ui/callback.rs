use std::rc::Rc;
use std::cell::RefCell;

use ui::Widget;

pub struct Callback<T> {
    cb: Rc<Fn(&T, &Rc<RefCell<Widget>>)>
}
//pub type Callback<T> = Rc<Fn(&T, &Rc<RefCell<Widget>>)>;

impl<T> Callback<T> {
    pub fn new(f: Rc<Fn(&T, &Rc<RefCell<Widget>>)>) -> Callback<T> {
        Callback {
            cb: f,
        }
    }

    pub fn with(f: Box<Fn()>) -> Callback<T> {
        Callback {
            cb: Rc::new(move |_t, _w| { f() }),
        }
    }

    pub fn remove_self() -> Callback<T> {
        Callback {
            cb: Rc::new(|_t, widget| { widget.borrow_mut().mark_for_removal() }),
        }
    }

    pub fn remove_parent() -> Callback<T> {
        Callback {
            cb: Rc::new(|_t, widget| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            }),
        }
    }

    pub fn call(&self, t: &T, widget: &Rc<RefCell<Widget>>) {
        (self.cb)(t, widget);
    }

    pub fn clone(&self) -> Callback<T> {
        Callback {
            cb: Rc::clone(&self.cb),
        }
    }
}
