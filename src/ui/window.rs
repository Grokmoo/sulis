use std::rc::Rc;
use std::cell::RefCell;

use resource::Point;
use ui::{Border, Button, Size, Label, Widget, WidgetKind};

pub struct Window {
    title_string: String,
}

impl Window {
    pub fn new(title: &str) -> Rc<Window> {
        Rc::new(Window {
            title_string: title.to_string(),
        })
    }
}

impl<'a> WidgetKind<'a> for Window {
    fn get_name(&self) -> &str {
        "Window"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        let mut label = Widget::with_position(Label::new(),
            Size::new(widget.borrow().state.size.width, 1),
            Point::from(&widget.borrow().state.position));
        Widget::set_text(&mut label, &self.title_string);

        let mut button = Widget::with_border(
            Button::new(Box::new(|widget, _state| {
                widget.parent.as_ref().unwrap();
                widget.parent.as_ref().unwrap().borrow().mark_for_removal()
            })),
            Size::new(3, 3),
            Point::new(widget.borrow().state.inner_right() - 3,
                widget.borrow().state.inner_top()),
            Border::as_uniform(1));
        Widget::set_background(&mut button, "background");
        Widget::set_text(&mut button, "x");

        vec![label, button]
    }
}
