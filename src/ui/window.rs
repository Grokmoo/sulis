use std::rc::Rc;
use std::cell::RefCell;

use resource::Point;
use ui::{Border, Button, Size, Label, Widget, WidgetKind};

pub struct Window<'a> {
    title_string: String,
    content: Rc<RefCell<Widget<'a>>>,
}

impl<'a> Window<'a> {
    pub fn new(title: &str, content: Rc<RefCell<Widget<'a>>>) -> Rc<Window<'a>> {
        Rc::new(Window {
            title_string: title.to_string(),
            content
        })
    }
}

impl<'a> WidgetKind<'a> for Window<'a> {
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
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            Size::new(3, 3),
            Point::new(widget.borrow().state.inner_right() - 3,
                widget.borrow().state.inner_top()),
            Border::as_uniform(1));
        Widget::set_background(&mut button, "background");
        Widget::set_text(&mut button, "x");

        vec![Rc::clone(&self.content), label, button]
    }
}
