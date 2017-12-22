use std::rc::Rc;
use std::cell::RefCell;

use ui::{Button, Size, Widget, WidgetKind};

pub struct ListBox {
    entries: Vec<String>,
}

impl ListBox {
    pub fn new(entries: Vec<String>) -> Rc<ListBox> {
        Rc::new(ListBox {
            entries
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
                Button::with_text(&entry, Box::new(|_w, _s| {  })),
                // TODO callback here that interfaces with a listbox overall callback
                // Button::with_text(&entry, Box::new(|_w, _s| info!("{}", entry))),
                "entry");
            children.push(label);
        }

        children
    }
}
