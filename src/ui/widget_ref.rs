use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

use ui::{Widget, WidgetBase};

//// A convenience struct for holding a 'Widget' and 'WidgetBase' object
//// together in a form where both can easily be referenced and modified
//// from the outside.  The main widget tree cannot use this struct as it
//// does not know the type of any individual widget in the tree.
pub struct WidgetRef<'a, T: Widget<'a>> {
    widget: Rc<RefCell<T>>,
    base: Rc<RefCell<WidgetBase<'a>>>,
}

impl<'a, T: Widget<'a>> WidgetRef<'a, T> {
    pub fn new(widget: Rc<RefCell<T>>,
               base: Rc<RefCell<WidgetBase<'a>>>) -> WidgetRef<'a, T> {

        WidgetRef {
            widget,
            base
        }
    }

    pub fn top(&self) -> Ref<T> {
        self.widget.borrow()
    }

    pub fn top_mut(&self) -> RefMut<T> {
        self.widget.borrow_mut()
    }

    pub fn base(&self) -> Ref<WidgetBase<'a>> {
        self.base.borrow()
    }

    pub fn base_mut(&self) -> RefMut<WidgetBase<'a>> {
        self.base.borrow_mut()
    }
}
