use std::rc::Rc;
use std::cmp;

use ui::theme::{HorizontalTextAlignment, VerticalTextAlignment};
use ui::{Widget, WidgetKind};
use io::{DrawList, TextRenderer};
use util::Point;

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

    fn get_draw_params(widget: &Widget) -> (i32, i32, &str) {
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

        (x, y, &text[0..len])
    }
}

impl WidgetKind for Label {
    fn get_name(&self) -> &str {
        "label"
    }

    fn layout(&self, widget: &mut Widget) {
        if let Some(ref text) = self.text {
            widget.state.add_text_param(text);
        }

        widget.do_base_layout();
    }

    fn get_draw_list(&self, widget: &Widget, _millis: u32) -> DrawList {
        let font = match &widget.state.font {
            &None => return DrawList::empty(),
            &Some(ref font) => font,
        };
        let (x, y, text) = Label::get_draw_params(widget);

        font.get_draw_list(text, &Point::new(x, y), 1.0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget,
                      _millis: u32) {
        let (x, y, text) = Label::get_draw_params(widget);
        renderer.set_cursor_pos(x, y);
        renderer.render_string(&text);
    }
}
