use std::rc::Rc;
use std::cmp;

use ui::{Widget, WidgetKind};
use io::TextRenderer;

pub struct Label {
}

impl Label {
    pub fn new() -> Rc<Label> {
        Rc::new(Label {
        })
    }
}

impl<'a> WidgetKind<'a> for Label {
    fn get_name(&self) -> &str {
        "Label"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget<'a>) {
        self.super_draw_text_mode(widget);

        let text = &widget.state.text;
        let x = widget.state.position.x;
        let y = widget.state.position.y;
        let w = widget.state.size.width;
        let h = widget.state.size.height;
        let len = cmp::min(text.len(), w as usize);

        let text = &text[0..len];

        let x = x + (w - len as i32) / 2;
        let y = y + (h - 1) / 2;
        let (max_x, max_y) = renderer.get_display_size();
        if x < 0 || y < 0 || x >= max_x || y >= max_y { return; }
        renderer.set_cursor_pos(x, y);
        renderer.render_string(&text);
    }
}
