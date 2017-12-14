use std::rc::Rc;

use resource::Point;
use ui::{Size, Label, Widget, WidgetKind};

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

    fn on_add(&self, widget: &mut Widget) {
       let mut label = Widget::with_position(Label::new(),
            Size::new(widget.state.size.width, 1),
            Point::from(&widget.state.position));
       label.state.set_text(&self.title_string);
       widget.add_child(label);
    }
}
