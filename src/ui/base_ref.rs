use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use ui::WidgetState;

pub struct BaseRef<'a> {
    base: Option<Rc<RefCell<WidgetState<'a>>>>,
}

impl<'a> BaseRef<'a> {
    pub fn new() -> BaseRef<'a> {
        BaseRef {
            base: None,
        }
    }

    pub fn set_base(&mut self, base: &Rc<RefCell<WidgetState<'a>>>) {
        self.base = Some(Rc::clone(base));
    }

    pub fn base(&self) -> Ref<WidgetState<'a>> {
        if let None = self.base {
            error!("Attempted ref in uninitialized BaseRef");
        }
        self.base.as_ref().unwrap().borrow()
    }

    pub fn base_mut(&self) -> RefMut<WidgetState<'a>> {
        if let None = self.base {
            error!("Attempted ref in uninitialized BaseRef");
        }

        self.base.as_ref().unwrap().borrow_mut()
    }
}
