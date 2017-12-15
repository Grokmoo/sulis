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
        let ref state = widget.borrow().state;

        let mut label = Widget::with_position(Label::new(),
            Size::new(state.size.width, 1),
            Point::from(&state.position));
        Widget::set_text(&mut label, &self.title_string);

        let center = Point::new(state.inner_left() + state.inner_size.width / 2,
                                state.inner_bottom() - 3);

        let mut cancel = Widget::with_border(
            Button::new(Box::new(|widget, _game_state| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            Size::new(10, 3),
            Point::new(center.x + 1, center.y),
            Border::as_uniform(1));
        Widget::set_background(&mut cancel, "background");
        Widget::set_text(&mut cancel, "Cancel");

        let mut quit = Widget::with_border(
            Button::new(Box::new(|_widget, game_state| { game_state.set_exit(); })),
            Size::new(10, 3),
            Point::new(center.x - 11, center.y),
            Border::as_uniform(1));
        Widget::set_background(&mut quit, "background");
        Widget::set_text(&mut quit, "Quit");

        vec![Rc::clone(&self.content), label, cancel, quit]
    }
}
