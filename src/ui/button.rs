use std::rc::Rc;
use std::cell::RefCell;

use ui::{Label, Widget, WidgetKind};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Rc<Label>,
    callback: Option<Box<Fn(&Rc<RefCell<Widget>>, &mut GameState)>>,
}

impl Button {
    pub fn empty() -> Rc<Button> {
        Rc::new(Button {
            label: Label::empty(),
            callback: None
        })
    }

    pub fn with_callback(callback: Box<Fn(&Rc<RefCell<Widget>>,
                                          &mut GameState)>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::empty(),
            callback: Some(callback)
        })
    }

    pub fn with_text(text: &str, callback: Box<Fn(&Rc<RefCell<Widget>>,
                                                  &mut GameState)>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(text),
            callback: Some(callback)
        })
    }
}

impl<'a> WidgetKind<'a> for Button {
    fn get_name(&self) -> &str {
        "button"
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
