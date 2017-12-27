use std::rc::Rc;
use std::cmp;

use ui::theme::{HorizontalTextAlignment, VerticalTextAlignment};
use ui::{Widget, WidgetKind};
use io::TextRenderer;

pub struct Label {
    pub text: Option<String>,
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

impl WidgetKind for Label {
    fn get_name(&self) -> &str {
        "label"
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();

        if let Some(ref text) = self.text {
            widget.state.set_text(&text);
        }
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget,
                      _millis: u32) {
        let text = &widget.state.text;
        let x = widget.state.inner_left();
        let y = widget.state.inner_top();
        let w = widget.state.inner_size.width;
        let h = widget.state.inner_size.height;
        let len = cmp::min(text.len(), w as usize);

        let x = match widget.state.horizontal_text_alignment {
            HorizontalTextAlignment::Left => x,
            HorizontalTextAlignment::Center => x + (w - len as i32) / 2,
            HorizontalTextAlignment::Right => x - len as i32,
        };

        let y = match widget.state.vertical_text_alignment {
            VerticalTextAlignment::Top => y,
            VerticalTextAlignment::Center => y + (h - 1) / 2,
            VerticalTextAlignment::Bottom => y + h - 1,
        };

        let text = &text[0..len];
        renderer.set_cursor_pos(x, y);
        renderer.render_string(&text);
    }
}
