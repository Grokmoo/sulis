use std::rc::Rc;
use std::str::FromStr;
use io::{DrawList, GraphicsRenderer, Vertex};
use resource::{Font, ResourceSet};
use ui::Color;
use ui::theme::TextParams;

pub trait FontRenderer {
    fn render(&self, renderer: &mut GraphicsRenderer, text: &str,
              pos_x: f32, pos_y: f32, defaults: &TextParams);

    fn get_font(&self) -> &Rc<Font>;
}

pub struct MarkupRenderer {
    font: Rc<Font>,
    width: f32,
}

impl MarkupRenderer {
    pub fn new(font: &Rc<Font>, width: i32) -> MarkupRenderer {
        // TODO pass the text, x, and y here.  all parsing can be
        // done at this stage and the DrawLists can be cached
        MarkupRenderer {
            font: Rc::clone(font),
            width: width as f32,
        }
    }
}

struct Markup {
    color: Color,
    scale: f32,
    pos_x: Option<f32>,
    pos_y: Option<f32>,
    image: Option<String>,
    font: Option<Rc<Font>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MarkupSetType {
    Color,
    Scale,
    PosX,
    PosY,
    Image,
    Font,
}

impl Markup {
    fn from_text_params(defaults: &TextParams) -> Markup {
        Markup {
            color: defaults.color,
            scale: defaults.scale,
            pos_x: None,
            pos_y: None,
            image: None,
            font: None,
        }
    }

    fn from_string(text: &str, defaults: &TextParams) -> Markup {
        let mut font: Option<Rc<Font>> = None;
        let mut image: Option<String> = None;
        let mut pos_x: Option<f32> = None;
        let mut pos_y: Option<f32> = None;
        let mut scale = defaults.scale;
        let mut color = defaults.color;
        let mut markup_set_type: Option<MarkupSetType> = None;
        let mut cur_buf = String::new();
        for c in text.chars() {
            match markup_set_type {
                None => markup_set_type = match c {
                    'c' => Some(MarkupSetType::Color),
                    's' => Some(MarkupSetType::Scale),
                    'x' => Some(MarkupSetType::PosX),
                    'y' => Some(MarkupSetType::PosY),
                    'i' => Some(MarkupSetType::Image),
                    'f' => Some(MarkupSetType::Font),
                    _ => None,
                }, Some(set_type) => match c {
                    '=' | ' ' => {
                        // skip
                    }, ';' => {
                        match set_type {
                            MarkupSetType::Color => color = get_color(&mut cur_buf),
                            MarkupSetType::Scale => scale = get_float(&mut cur_buf),
                            MarkupSetType::PosX => pos_x = Some(get_float(&mut cur_buf)),
                            MarkupSetType::PosY => pos_y = Some(get_float(&mut cur_buf)),
                            MarkupSetType::Image => image = get_str(&mut cur_buf),
                            MarkupSetType::Font => font = get_font(&mut cur_buf),
                        }
                        markup_set_type = None;
                    }, _ => {
                        cur_buf.push(c);
                    }
                }
            }
        }

        if let Some(set_type) = markup_set_type {
            match set_type {
                MarkupSetType::Color => color = get_color(&mut cur_buf),
                MarkupSetType::Scale => scale = get_float(&mut cur_buf),
                MarkupSetType::PosX => pos_x = Some(get_float(&mut cur_buf)),
                MarkupSetType::PosY => pos_y = Some(get_float(&mut cur_buf)),
                MarkupSetType::Image => image = get_str(&mut cur_buf),
                MarkupSetType::Font => font = get_font(&mut cur_buf),
            }
        }

        Markup {
            color,
            scale,
            pos_x,
            pos_y,
            image,
            font,
        }
    }
}

fn get_str(buf: &mut String) -> Option<String> {
    let string = Some(buf.to_string());
    buf.clear();
    string
}

fn get_font(buf: &mut String) -> Option<Rc<Font>> {
    let font = ResourceSet::get_font(buf);
    buf.clear();
    font
}

fn get_float(buf: &mut String) -> f32 {
    let scale = f32::from_str(buf);
    buf.clear();
    match scale {
        Err(_) => {
            warn!("Unable to parse float from format string '{}'", buf);
            1.0
        },
        Ok(scale) => scale,
    }
}

fn get_color(buf: &mut String) -> Color {
    let color = Color::from_string(buf);
    buf.clear();
    color
}

fn get_y_offset(line_height: f32, font: &Rc<Font>) -> f32 {
    (line_height - 1.0) * font.base as f32 / font.line_height as f32
}

fn draw_current(renderer: &mut GraphicsRenderer, font_id: &str,
                quads: Vec<[Vertex; 4]>, markup: &Markup) {
        let mut draw_list = DrawList::from_font(font_id, quads);
        draw_list.set_color(markup.color);
        renderer.draw(draw_list);
}

fn draw_sprite(renderer: &mut GraphicsRenderer, image: &str,
               markup: &Markup, x: f32, y: f32) {
    let sprite = match ResourceSet::get_sprite_from(image) {
        None => {
            warn!("Unable to find image '{}'", image);
            return;
        },
        Some(sprite) => sprite,
    };

    let x_over_y = sprite.size.width as f32 / sprite.size.height as f32;
    let mut draw_list = DrawList::from_sprite_f32(&sprite, x, y,
                                                  markup.scale * x_over_y, markup.scale);
    draw_list.set_color(markup.color);
    renderer.draw(draw_list);
}

impl FontRenderer for MarkupRenderer {
    fn render(&self, renderer: &mut GraphicsRenderer, text: &str, pos_x: f32, pos_y: f32,
              defaults: &TextParams) {
        let line_height = defaults.scale;
        let max_x = pos_x + self.width;
        let mut quads: Vec<[Vertex; 4]> = Vec::new();
        let mut x = pos_x;
        let mut y = pos_y;
        let mut cur_font = Rc::clone(&self.font);

        let mut in_markup_tag = false;
        let mut cur_markup = Markup::from_text_params(defaults);
        let mut markup_buf = String::new();
        let mut max_last_line_height = cur_markup.scale;
        let mut y_offset = get_y_offset(cur_markup.scale, &self.font);
        for c in text.chars() {
            match c {
                '[' => {
                    draw_current(renderer, &cur_font.id, quads, &cur_markup);
                    quads = Vec::new();
                    in_markup_tag = true;
                }, '|' => {
                    in_markup_tag = false;
                    cur_markup = Markup::from_string(&markup_buf, &defaults);
                    markup_buf.clear();
                    if let Some(ref font) = cur_markup.font {
                        cur_font = Rc::clone(font);
                    }
                    if let Some(markup_x) = cur_markup.pos_x {
                        x = pos_x + markup_x;
                    }
                    if let Some(markup_y) = cur_markup.pos_y {
                        y = pos_y + markup_y;
                        max_last_line_height = cur_markup.scale;
                    }
                    if let Some(ref image) = cur_markup.image {
                        draw_sprite(renderer, &image, &cur_markup, x, y);
                    }
                    y_offset = get_y_offset(cur_markup.scale, &cur_font);
                    if cur_markup.scale > max_last_line_height {
                        max_last_line_height = cur_markup.scale;
                    }
                }, ']' => {
                    draw_current(renderer, &cur_font.id, quads, &cur_markup);
                    cur_font = Rc::clone(&self.font);
                    quads = Vec::new();
                    cur_markup = Markup::from_text_params(defaults);
                    y_offset = get_y_offset(cur_markup.scale, &cur_font);
                    if cur_markup.scale > max_last_line_height {
                        max_last_line_height = cur_markup.scale;
                    }
                }, '\n' => {
                    x = pos_x;
                    y += max_last_line_height;
                    max_last_line_height = line_height;
                }, _ => {
                    if in_markup_tag {
                        markup_buf.push(c);
                    } else {
                        x = cur_font.get_quad(&mut quads, c, x, y - y_offset, cur_markup.scale);
                    }
                }
            }

            if x > max_x {
                x = pos_x;
                y += max_last_line_height;
                max_last_line_height = cur_markup.scale;
            }
        }

        let mut draw_list = DrawList::from_font(&cur_font.id, quads);
        draw_list.set_color(defaults.color);
        renderer.draw(draw_list);
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
