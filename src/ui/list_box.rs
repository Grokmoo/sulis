use std::rc::Rc;
use std::cell::RefCell;

use ui::{Button, Callback, Size, Widget, WidgetKind};

pub struct ListBox {
    entries: Vec<String>,
    callback: Option<Callback>,
}

impl ListBox {
    pub fn new(entries: Vec<String>) -> Rc<ListBox> {
        Rc::new(ListBox {
            entries,
            callback: None,
        })
    }

    pub fn with_callback(callback: Callback, entries: Vec<String>) -> Rc<ListBox> {
        Rc::new(ListBox {
            entries,
            callback: Some(callback),
        })
    }
}

impl<'a> WidgetKind<'a> for ListBox {
    fn get_name(&self) -> &str {
        "list_box"
    }

    fn layout(&self, widget: &mut Widget<'a>) {
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

    fn on_add(&self, _widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {

        let mut children: Vec<Rc<RefCell<Widget<'a>>>> =
            Vec::with_capacity(self.entries.len());

        for entry in self.entries.iter() {
            let label = Widget::with_theme(
                // TODO pass a callback here
                //Button::new(&entry, self.callback), "entry");
                Button::with_text(&entry), "entry");
            children.push(label);
        }

        children
    }
}
