use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use ui::{Widget, WidgetBase};
use io::TextRenderer;

pub struct Label {
    text: Option<String>,
}

impl Label {
    pub fn new(text: &str) -> Rc<RefCell<Label>> {
        Rc::new(RefCell::new(Label {
            text: Some(text.to_string()),
        }))
    }

    pub fn new_empty() -> Rc<RefCell<Label>> {
        Rc::new(RefCell::new(Label {
            text: None
        }))
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = Some(text.to_string());
    }

    pub fn clear_text(&mut self) {
        self.text = None;
    }
}

impl Widget for Label {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, owner: &WidgetBase) {
        if let Some(ref t) = self.text {
            let x = owner.x;
            let y = owner.y;
            let w = owner.width;
            let len = cmp::min(t.len(), w as usize);

            let text = &t[0..len];
            renderer.set_cursor_pos(x + (w - len as i32) / 2, y);
            renderer.render_string(&text);
        }
    }
}
