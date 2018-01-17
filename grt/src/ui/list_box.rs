use std::fmt::Display;
use std::rc::Rc;
use std::slice::Iter;
use std::cell::RefCell;

use ui::{AnimationState, Button, Callback, Widget, WidgetKind};
use util::Size;

pub struct Entry<T: Display> {
    item: T,
    callback: Option<Callback>,
    animation_state: AnimationState,
}

impl<T: Display> Entry<T> {
    pub fn item(&self) -> &T {
        &self.item
    }
}

impl<T: Display> Entry<T> {
    pub fn new(item: T, callback: Option<Callback>) -> Entry<T> {
       Entry {
           item,
           callback,
           animation_state: AnimationState::default(),
       }
    }

    pub fn with_state(item: T, callback: Option<Callback>,
                      animation_state: AnimationState) -> Entry<T> {
        Entry {
            item,
            callback,
            animation_state,
        }
    }
}

pub struct ListBox<T: Display> {
    entries: Vec<Entry<T>>,
}

impl<T: Display> ListBox<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<ListBox<T>> {
        Rc::new(ListBox {
            entries,
        })
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries.get(index)
    }
}

impl<T: Display> WidgetKind for ListBox<T> {
    fn get_name(&self) -> &str {
        "list_box"
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_self_layout();

        let width = widget.state.inner_size.width;
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            let theme = match child.borrow().theme {
                None => continue,
                Some(ref t) => Rc::clone(t),
            };
            let height = theme.preferred_size.height;
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
        }
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children: Vec<Rc<RefCell<Widget>>> =
            Vec::with_capacity(self.entries.len());

        for entry in self.entries.iter() {
            let widget = Widget::with_theme(Button::with_text(&entry.item.to_string()), "entry");
            widget.borrow_mut().state.set_animation_state(&entry.animation_state);
            if let Some(ref cb) = entry.callback {
                widget.borrow_mut().state.add_callback(cb.clone());
            }
            children.push(widget);
        }

        children
    }
}
