use std::rc::Rc;

use ui::{MarkupRenderer, Widget, WidgetKind};
use io::GraphicsRenderer;
use util::Point;

pub struct TextArea {
    pub text: Option<String>,
}

impl TextArea {
    pub fn empty() -> Rc<TextArea> {
        Rc::new(TextArea {
            text: None,
        })
    }

    pub fn new(text: &str) -> Rc<TextArea> {
        Rc::new(TextArea {
            text: Some(text.to_string()),
        })
    }
}

impl WidgetKind for TextArea {
    fn get_name(&self) -> &str {
        "text_area"
    }

    fn layout(&self, widget: &mut Widget) {
        if let Some(ref text) = self.text {
            widget.state.add_text_arg("0", text);
        }

        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(
                    MarkupRenderer::new(font, widget.state.inner_size.width)
                    ));
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, _millis: u32) {
        let font_rend = match &widget.state.text_renderer {
            &None => return,
            &Some(ref renderer) => renderer,
        };

        let scale = widget.state.text_params.scale;
        let x = widget.state.inner_left() as f32;
        let y = widget.state.inner_top() as f32;

        let mut draw_list = font_rend.render(&widget.state.text, x, y, scale);
        draw_list.set_color(widget.state.text_params.color);

        renderer.draw(draw_list);
    }
}
