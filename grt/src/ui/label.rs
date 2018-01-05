use std::rc::Rc;
use std::cmp;

use ui::theme::{HorizontalTextAlignment, VerticalTextAlignment};
use ui::{Widget, WidgetKind};
use io::{self, DrawList, TextRenderer};

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

    fn get_draw_lists(&self, widget: &Widget, _millis: u32) -> Vec<DrawList> {
        let font = match &widget.state.font {
            &None => return Vec::new(),
            &Some(ref font) => font,
        };
        let (x, y, text) = Label::get_draw_params(widget);
        let b_scale_l = io::GFX_BORDER_SCALE * widget.state.border.left as f32;
        let _b_scale_r = io::GFX_BORDER_SCALE * widget.state.border.right as f32;
        let b_scale_t = io::GFX_BORDER_SCALE * widget.state.border.top as f32;
        let b_scale_b = io::GFX_BORDER_SCALE * widget.state.border.bottom as f32;

        vec![font.get_draw_list(text, x as f32 - b_scale_l,
                                y as f32 - b_scale_t,
                                1.0 + b_scale_t + b_scale_b)]
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget,
                      _millis: u32) {
        let (x, y, text) = Label::get_draw_params(widget);
        renderer.set_cursor_pos(x, y);
        renderer.render_string(&text);
    }
}
