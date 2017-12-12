use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use ui::WidgetBase;

pub struct BaseRef<'a> {
    base: Option<Rc<RefCell<WidgetBase<'a>>>>,
}

impl<'a> BaseRef<'a> {
    pub fn new() -> BaseRef<'a> {
        BaseRef {
            base: None,
        }
    }

    pub fn set_base(&mut self, base: &Rc<RefCell<WidgetBase<'a>>>) {
        self.base = Some(Rc::clone(base));
    }

    pub fn base(&self) -> Ref<WidgetBase<'a>> {
        if let None = self.base {
            error!("Attempted ref in uninitialized BaseRef");
        }
        self.base.as_ref().unwrap().borrow()
    }

    pub fn base_mut(&self) -> RefMut<WidgetBase<'a>> {
        if let None = self.base {
            error!("Attempted ref in uninitialized BaseRef");
        }

        self.base.as_ref().unwrap().borrow_mut()
    }
}
