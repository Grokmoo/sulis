use std::rc::Rc;
use std::cell::RefCell;

use ui::Widget;

pub struct Callback {
    cb: Rc<Fn(&Rc<RefCell<Widget>>)>
}

impl Callback {
    pub fn new(f: Rc<Fn(&Rc<RefCell<Widget>>)>) -> Callback {
        Callback {
            cb: f,
        }
    }

    pub fn with(f: Box<Fn()>) -> Callback {
        Callback {
            cb: Rc::new(move |_w| { f() }),
        }
    }

    pub fn remove_self() -> Callback {
        Callback {
            cb: Rc::new(|widget| { widget.borrow_mut().mark_for_removal() }),
        }
    }

    pub fn remove_parent() -> Callback {
        Callback {
            cb: Rc::new(|widget| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            }),
        }
    }

    pub fn call(&self, widget: &Rc<RefCell<Widget>>) {
        (self.cb)(widget);
    }

    pub fn clone(&self) -> Callback {
        Callback {
            cb: Rc::clone(&self.cb),
        }
    }
}
