use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use ui::{Widget, WidgetBase, BaseRef};
use io::TextRenderer;

pub struct Label<'a> {
    pub(in ::ui) text: Option<String>,
    pub base_ref: BaseRef<'a>,
}

impl<'a> Label<'a> {
    pub fn new(text: &str) -> Rc<RefCell<Label<'a>>> {
        Rc::new(RefCell::new(Label {
            text: Some(text.to_string()),
            base_ref: BaseRef::new(),
        }))
    }

    pub fn new_empty() -> Rc<RefCell<Label<'a>>> {
        Rc::new(RefCell::new(Label {
            text: None,
            base_ref: BaseRef::new(),
        }))
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = Some(text.to_string());
    }

    pub fn clear_text(&mut self) {
        self.text = None;
    }
}

impl<'a> Widget<'a> for Label<'a> {
    fn get_name(&self) -> &str {
        "Label"
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<WidgetBase<'a>>>) {
        self.base_ref.set_base(parent);
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        if let Some(ref t) = self.text {
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
        }
    }
}
