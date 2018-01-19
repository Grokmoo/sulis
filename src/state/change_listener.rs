use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::Widget;

pub struct ChangeListenerList<T> {
   listeners: Vec<ChangeListener<T>>,
}

impl<T> Default for ChangeListenerList<T> {
    fn default() -> ChangeListenerList<T> {
        ChangeListenerList {
            listeners: Vec::new(),
        }
    }
}

impl<T> ChangeListenerList<T> {
    pub fn add(&mut self, listener: ChangeListener<T>) {
        self.remove(listener.id);
        self.listeners.push(listener);
    }

    pub fn remove(&mut self, id: &'static str) {
        self.listeners.retain(|listener| listener.id() != id);
    }

    pub fn notify(&self, t: &T) {
        for listener in self.listeners.iter() {
            listener.call(t);
        }
    }
}

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

    pub fn remove_widget(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().mark_for_removal();
            }),
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

    pub fn invalidate_layout(id: &'static str, widget: &Rc<RefCell<Widget>>) -> ChangeListener<T> {
        let widget_ref = Rc::clone(widget);
        ChangeListener {
            cb: Box::new(move |_t| {
                widget_ref.borrow_mut().invalidate_layout();
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
