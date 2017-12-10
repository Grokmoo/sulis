use std::rc::Rc;
use std::cell::RefCell;

use ui::{Label, Widget, WidgetBase};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Label,
    callback: Box<FnMut(&mut WidgetBase, &mut GameState)>,
}

impl Button {
    pub fn new(text: &str) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label: Label {
                text: Some(text.to_string()),
            },
            callback: {
                Box::new(|_parent, _state| debug!("hello"))
            }
        }))
    }
}

impl<'a> Widget<'a> for Button {
    fn get_name(&self) -> &str {
        "Button"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, owner: &WidgetBase) {
        self.label.draw_text_mode(renderer, owner);
    }

    fn on_mouse_click(&mut self, parent: &mut WidgetBase, state: &mut GameState,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        (self.callback)(parent, state);
        true
    }
}
