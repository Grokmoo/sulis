use std::rc::Rc;
use std::cell::RefCell;

use ui::{AnimationState, Button, Callback, Size, Widget, WidgetKind};

pub struct Entry {
    text: String,
    callback: Option<Callback<Button>>,
    animation_state: AnimationState,
}

impl Entry {
    pub fn new(text: &str, callback: Option<Callback<Button>>) -> Entry {
       Entry {
           text: text.to_string(),
           callback,
           animation_state: AnimationState::Base,
       }
    }

    pub fn with_state(text: &str, callback: Option<Callback<Button>>,
                      animation_state: AnimationState) -> Entry {
        Entry {
            text: text.to_string(),
            callback,
            animation_state,
        }
    }
}

pub struct ListBox {
    entries: Vec<Entry>,
}

impl ListBox {
    pub fn new(entries: Vec<Entry>) -> Rc<ListBox> {
        Rc::new(ListBox {
            entries,
        })
    }
}

impl WidgetKind for ListBox {
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
            let kind = match &entry.callback {
                &Some(ref cb) => Button::new(&entry.text, Rc::clone(&cb)),
                &None => Button::with_text(&entry.text),
            };

            let widget = Widget::with_theme(kind, "entry");
            widget.borrow_mut().state.set_animation_state(entry.animation_state);
            children.push(widget);
        }

        children
    }
}
