use std::rc::Rc;
use io::{DrawList, GraphicsRenderer, Vertex};
use resource::Font;
use ui::theme::TextParams;

pub trait FontRenderer {
    fn render(&self, renderer: &mut GraphicsRenderer, text: &str,
              pos_x: f32, pos_y: f32, defaults: &TextParams);

    fn get_font(&self) -> &Rc<Font>;
}

pub struct LineRenderer {
    font: Rc<Font>,
}

impl LineRenderer {
    pub fn new(font: &Rc<Font>) -> LineRenderer {
        LineRenderer {
            font: Rc::clone(font),
        }
    }
}

impl FontRenderer for LineRenderer {
    fn render(&self, renderer: &mut GraphicsRenderer, text: &str, pos_x: f32, pos_y: f32,
              defaults: &TextParams) {
        let mut quads: Vec<[Vertex; 4]> = Vec::new();
        let mut x = pos_x;
        for c in text.chars() {
            x = self.font.get_quad(&mut quads, c, x, pos_y, defaults.scale);
        }

        let mut draw_list = DrawList::from_font(&self.font.id, quads);
        draw_list.set_color(defaults.color);
        renderer.draw(draw_list);
    }

    fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
}
