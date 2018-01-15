use std::rc::Rc;
use std::cell::RefCell;

use ui::{Label, LineRenderer, Widget, WidgetKind};
use io::{event, GraphicsRenderer, TextRenderer};
use util::Point;

pub struct Button {
    label: Rc<Label>,
}

impl Button {
    pub fn empty() -> Rc<Button> {
        Rc::new(Button {
            label: Label::empty(),
        })
    }

    pub fn with_text(text: &str) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(text),
        })
    }
}

impl WidgetKind for Button {
    fn get_name(&self) -> &str {
        "button"
    }

    fn layout(&self, widget: &mut Widget) {
        if let Some(ref text) = self.label.text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      widget: &Widget, millis: u32) {
        self.label.draw_text_mode(renderer, widget, millis);
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.label.draw_graphics_mode(renderer, pixel_size, widget, millis);
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        Widget::fire_callback(widget);
        true
    }
}
