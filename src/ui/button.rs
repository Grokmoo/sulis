use std::rc::Rc;
use std::cell::RefCell;

use ui::{Callback, Label, Widget, WidgetKind};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Rc<Label>,
    callback: Option<Callback>,
}

impl Button {
    pub fn new(text: &str, callback: Option<Callback>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(text),
            callback,
        })
    }

    pub fn with_callback(callback: Callback) -> Rc<Button> {
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

impl<'a> WidgetKind<'a> for Button {
    fn get_name(&self) -> &str {
        "button"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        if let Some(ref text) = self.label.text {
            widget.borrow_mut().state.set_text(&text);
        }

        Vec::with_capacity(0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget<'a>) {
        self.label.draw_text_mode(renderer, widget);
    }

    fn on_mouse_click(&self, state: &mut GameState, widget: &Rc<RefCell<Widget<'a>>>,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        match self.callback {
            Some(ref cb) => (cb)(widget, state),
            None => (),
        };
        true
    }
}
