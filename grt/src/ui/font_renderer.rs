use std::rc::Rc;

use io::{DrawList, Vertex};
use resource::Font;

pub trait FontRenderer {
    fn render(&self, text: &str, pos_x: f32, pos_y: f32, line_height: f32) -> DrawList;

    fn get_font(&self) -> &Rc<Font>;
}

pub struct MarkupRenderer {
    font: Rc<Font>,
    width: f32,
}

impl MarkupRenderer {
    pub fn new(font: &Rc<Font>, width: i32) -> MarkupRenderer {
        MarkupRenderer {
            font: Rc::clone(font),
            width: width as f32,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum MarkupTag {
    H1,
    H2,
    H3,
    H4,
}

fn get_markup_tag(text: &str) -> Option<MarkupTag> {
    use self::MarkupTag::*;
    let tag = match text {
        "h1" => H1,
        "h2" => H2,
        "h3" => H3,
        "h4" => H4,
        _ => return None,
    };
    Some(tag)
}

fn get_line_height(tag: &Option<MarkupTag>) -> f32 {
    use self::MarkupTag::*;
    if let &Some(tag) = tag {
        match tag {
            H1 => 2.0,
            H2 => 1.75,
            H3 => 1.5,
            H4 => 1.25,
        }
    } else {
    1.0
    }
}

fn get_y_offset(line_height: f32, font: &Rc<Font>) -> f32 {
    (line_height - 1.0) * font.base as f32 / font.line_height as f32
}

impl FontRenderer for MarkupRenderer {
    fn render(&self, text: &str, pos_x: f32, pos_y: f32, line_height: f32) -> DrawList {
        let max_x = pos_x + self.width;
        let mut quads: Vec<[Vertex; 4]> = Vec::new();
        let mut x = pos_x;
        let mut y = pos_y;

        let mut in_markup_tag = false;
        let mut cur_markup_tag: Option<MarkupTag> = None;
        let mut markup_tag_buf = String::new();
        let mut cur_line_height = line_height;
        let mut max_last_line_height = line_height;
        let mut y_offset = get_y_offset(cur_line_height, &self.font);
        for c in text.chars() {
            match c {
                '[' => {
                    in_markup_tag = true;
                }, ']' => {
                    in_markup_tag = false;
                    cur_markup_tag = get_markup_tag(&markup_tag_buf);
                    markup_tag_buf.clear();
                }, '{' => {
                    cur_line_height = get_line_height(&cur_markup_tag);
                    y_offset = get_y_offset(cur_line_height, &self.font);
                    if cur_line_height > max_last_line_height {
                        max_last_line_height = cur_line_height;
                    }
                }, '}' => {
                    cur_line_height = line_height;
                    y_offset = get_y_offset(cur_line_height, &self.font);
                }, '\n' => {
                    x = pos_x;
                    y += max_last_line_height;
                    max_last_line_height = line_height;
                }, _ => {
                    if in_markup_tag {
                        markup_tag_buf.push(c);
                    } else {
                        x = self.font.get_quad(&mut quads, c, x, y - y_offset, cur_line_height);
                    }
                }
            }

            if x > max_x {
                x = pos_x;
                y += max_last_line_height;
                max_last_line_height = line_height;
            }
        }

        DrawList::from_font(&self.font.id, quads)
    }

    fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
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
    fn render(&self, text: &str, pos_x: f32, pos_y: f32, line_height: f32) -> DrawList {
        let mut quads: Vec<[Vertex; 4]> = Vec::new();
        let mut x = pos_x;
        for c in text.chars() {
            x = self.font.get_quad(&mut quads, c, x, pos_y, line_height);
        }

        DrawList::from_font(&self.font.id, quads)
    }

    fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
}
