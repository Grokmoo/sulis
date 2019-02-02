//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::rc::Rc;
use std::str::FromStr;

use crate::io::Vertex;
use crate::resource::{Font, ResourceSet};
use crate::ui::{self, Color, WidgetState};
use crate::ui::theme::TextParams;

#[derive(Debug, PartialEq, Copy, Clone)]
enum MarkupKind {
    Center,
    Right,
    Color,
    Scale,
    PosX,
    PosY,
    Image,
    Font,
    If,
    IfNot,
}

pub struct Markup {
    pub color: Color,
    pub scale: f32,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub image: Option<String>,
    pub center: Option<f32>,
    pub right: Option<f32>,
    pub font: Rc<Font>,
    pub ignore: bool,
}

impl Markup {
    pub fn from_text_params(defaults: &TextParams, font: &Rc<Font>) -> Markup {
        Markup {
            color: defaults.color,
            scale: defaults.scale,
            pos_x: None,
            pos_y: None,
            image: None,
            center: None,
            right: None,
            font: Rc::clone(font),
            ignore: false,
        }
    }

    pub fn from_other(other: &Markup) -> Markup {
        Markup {
            color: other.color,
            scale: other.scale,
            pos_x: None,
            pos_y: None,
            image: None,
            center: None,
            right: None,
            font: Rc::clone(&other.font),
            ignore: other.ignore,
        }
    }

    pub fn from_string(text: &str, defaults: &Markup, widget_state: &WidgetState) -> Markup {
        let mut markup = Markup::from_other(defaults);
        let mut markup_kind: Option<MarkupKind> = None;
        let mut cur_buf = String::new();
        for c in text.chars() {
            use self::MarkupKind::*;
            match markup_kind {
                None => markup_kind = match c {
                    'r' => Some(Right),
                    'a' => Some(Center),
                    'c' => Some(Color),
                    's' => Some(Scale),
                    'x' => Some(PosX),
                    'y' => Some(PosY),
                    'i' => Some(Image),
                    'f' => Some(Font),
                    '?' => Some(If),
                    '!' => Some(IfNot),
                    _ => None,
                }, Some(kind) => match c {
                    '=' | ' ' => {
                        // skip
                    }, ';' => {
                        markup.parse_buf(&cur_buf, kind, widget_state);
                        cur_buf.clear();
                        markup_kind = None;
                    }, _ => {
                        cur_buf.push(c);
                    }
                }
            }
        }

        if let Some(kind) = markup_kind {
            markup.parse_buf(&cur_buf, kind, widget_state);
        }
        markup
    }

    pub fn add_quad_and_advance(&self, quads: &mut Vec<Vertex>, c: char, x: f32, y: f32) -> f32 {
        if self.ignore { return x; }

        self.font.get_quad(quads, c, x, y - self.y_offset(), self.scale)
    }

    fn y_offset(&self) -> f32 {
        (self.scale - 1.0) * self.font.base as f32 / self.font.line_height as f32
    }

    fn parse_buf(&mut self, buf: &str, kind: MarkupKind, widget_state: &WidgetState) {
        use self::MarkupKind::*;
        match kind {
            Color => self.color = ui::Color::from_string(buf),
            Scale => self.scale = get_float(buf),
            PosX => self.pos_x = Some(get_float(buf)),
            PosY => self.pos_y = Some(get_float(buf)),
            Center => self.center = Some(get_float(buf)),
            Right => self.right = Some(get_float(buf)),
            Image => self.image = Some(buf.to_string()),
            Font => match ResourceSet::font(buf) {
                None => warn!("Font not found '{}'", buf),
                Some(font) => self.font = font,
            },
            If => self.ignore = self.ignore || !widget_state.has_text_arg(buf),
            IfNot => self.ignore = self.ignore || widget_state.has_text_arg(buf),
        }
    }
}

fn get_float(buf: &str) -> f32 {
    let val = f32::from_str(buf);
    match val {
        Err(_) => {
            warn!("Unable to parse float from format string '{}'", buf);
            1.0
        },
        Ok(val) => val,
    }
}

