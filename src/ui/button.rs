use std::rc::Rc;
use std::cell::RefCell;

use ui::{Callback, Label, Widget, WidgetKind};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Rc<Label>,
    callback: Option<Callback<Button>>,
}

impl Button {
    pub fn new(text: &str, callback: Callback<Button>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(text),
            callback: Some(callback),
        })
    }

    pub fn with_callback(callback: Callback<Button>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::empty(),
            callback: Some(callback)
        })
    }

    pub fn with_text(text: &str) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(text),
            callback: None
        })
    }
}

impl WidgetKind for Button {
    fn get_name(&self) -> &str {
        "button"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        if let Some(ref text) = self.label.text {
            widget.borrow_mut().state.set_text(&text);
        }

        Vec::with_capacity(0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget) {
        self.label.draw_text_mode(renderer, widget);
    }

    fn on_mouse_click(&self, state: &mut GameState, widget: &Rc<RefCell<Widget>>,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        match self.callback {
            Some(ref cb) => (cb)(self, widget, state),
            None => (),
        };
        true
    }
}
