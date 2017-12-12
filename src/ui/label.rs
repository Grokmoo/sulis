use std::rc::Rc;

use ui::WidgetKind;
use io::TextRenderer;

pub struct Label {
    pub(in ::ui) text: Option<String>,
}

impl Label {
    pub fn new(text: &str) -> Rc<Label> {
        Rc::new(Label {
            text: Some(text.to_string()),
        })
    }

    pub fn new_empty() -> Rc<Label> {
        Rc::new(Label {
            text: None,
        })
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = Some(text.to_string());
    }

    pub fn clear_text(&mut self) {
        self.text = None;
    }
}

impl<'a> WidgetKind<'a> for Label {
    fn get_name(&self) -> &str {
        "Label"
    }

    fn draw_text_mode(&self, _renderer: &mut TextRenderer) {
        //if let Some(ref t) = self.text {
            // let base = self.base_ref.base();
            // let x = base.position.x;
            // let y = base.position.y;
            // let w = base.size.width;
            // let h = base.size.height;
            // let len = cmp::min(t.len(), w as usize);
            //
            // let text = &t[0..len];
            //
            // let x = x + (w - len as i32) / 2;
            // let y = y + (h - 1) / 2;
            // let (max_x, max_y) = renderer.get_display_size();
            // if x < 0 || y < 0 || x >= max_x || y >= max_y { return; }
            // renderer.set_cursor_pos(x, y);
            // renderer.render_string(&text);
       // }
    }
}
