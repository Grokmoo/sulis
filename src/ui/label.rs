use std::rc::Rc;
use std::cmp;

use ui::{Widget, WidgetKind};
use io::TextRenderer;

pub struct Label {
    text: Option<String>,
}

impl Label {
    pub fn empty() -> Rc<Label> {
        Rc::new(Label {
            text: None,
        })
    }

    pub fn new(text: &str) -> Rc<Label> {
        Rc::new(Label {
            text: Some(text.to_string()),
        })
    }
}

impl<'a> WidgetKind<'a> for Label {
    fn get_name(&self) -> &str {
        "label"
    }

    fn layout(&self, widget: &mut Widget<'a>) {
        widget.do_base_layout();

        if let Some(ref text) = self.text {
            widget.state.set_text(&text);
        }
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget<'a>) {
        let text = &widget.state.text;
        let x = widget.state.position.x;
        let y = widget.state.position.y;
        let w = widget.state.size.width;
        let h = widget.state.size.height;
        let len = cmp::min(text.len(), w as usize);

        let text = &text[0..len];
        let x = x + (w - len as i32) / 2;
        let y = y + (h - 1) / 2;
        renderer.set_cursor_pos(x, y);
        renderer.render_string(&text);
    }
}
