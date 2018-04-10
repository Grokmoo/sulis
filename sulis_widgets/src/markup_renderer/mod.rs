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

//! A font renderer that parses a simple markup language.  This
//! allows for fairly complex document structure to be displayed
//! inside various widgets, particularly `TextArea`.  The text
//! itself is specified in the theme.  The text_params attribute
//! in the theme is used as the defaults.
//!
//! The markup is parsed very loosely - invalid markup may give the
//! results you were intending, something different, or no rendering
//! at all.  It may output warnings.  Invalid markup should not ever
//! cause any panics or other problems outside the scope of the text
//! being rendered.
//!
//! Note that text arguments using #arg# are parsed separately, and
//! are expanded before this markup is parsed.  See `WidgetState#add_text_arg`
//!
//! # Example
//! [s=2.0|s specifies a scaling factor]
//! [s=2.0;c=00ff00|Use ; to add multiple params to one tag]
//! [s=2.0;c=00ff00|You can also [c=00ffff|nest] tags]
//! [s=1.5|This is a smaller size tag spanning
//! multiple lines]
//!
//! Line size is based on the [s=1.25|largest] size in the line.
//!
//! [c=00ff00ff|All of] [c=00ff00|these are] [c=0f0f|the same] [c=0f0|color]
//!
//! Some characters need to be escaped \\, \[, \], \|
//!
//! [x=0;y=10|Using x or y causes the writing to be repositioned to that
//! location] Rendering will continue as normal after that point; the
//! position is not set back to where it was before.  This can be used
//! to make simple tables:
//!
//! [x=0|Table col][x=10.5|Column 2]
//! [x=0|Second row][x=10.5|Row 2, Col 2]
//!
//! [f=mono|You can specify another font]
//! [i=spritesheet/sprite;s=5.0|] [y=20|]You can embed images.  You'll probably need to
//! set the write position before and after.
//!
//! # Tag Format
//! Tags begin with [.  Then, in the first section, one or more params should be
//! specified.  After all params are specified, use the | character to move to the
//! next section.  In this section is optional text that the params will be applied
//! to.  Close the tag with ].
//!
//! # List of params
//! * **c** - Specify a color, in one of several formats, all hex based:
//! `RRGGBBAA`, `RRGGBB`, `RGBA`, `RGB`.  When using 2 characters for a component,
//! you are specifying with full byte precision.  When using 1 character, you are
//! specifying the 4 most significant bits.
//! * **s** - Specify a size as a float, with 1.0 being the basic text size.  The
//! decimal part of the float is optional.
//! * **x** - Causes writing to be repositioned to the given x coordinate.  This
//! is not reset after the tag, so `[x=10|Some text]` and `[x=10|]Some text` are
//! equivalent.
//! * **y** - Causes writing to be repositioned to the given y coordinate, in the
//! same manner as `x` above.
//! * **i** - Embeds an image.  The image must be referenced as `spritesheet/sprite`
//! Note that drawing an image does not advance the writing cursor.  You will probably
//! want to scale your image with `s`
//! * **f** - Writes using another defined font.
//! * **?** - Checks for the existance of a text argument.  If the argument is not
//! present, this tag is ignored when producing the output.
//! * **!** - Checks for the existance of a text argument.  If the argument is present,
//! this tag is ignored when producing the output.
//! # Line Wrapping
//! The character '\n' is treated as a line break, and causes wrap around to the
//! next line.  Lines that are too long will also be wrapped, with basic whitespace
//! splitting.  You can preserve line break characters in yaml using '|', i.e:
//!
//! text: |
//!   Some long text with preserved line breaks.
//!   The base indentation level is stripped out by the YAML parser.
//!
//! # Escape Characters
//! Use '\' to generate an escape character.  The next character will be directly
//! output instead of potentially parsed as a tag.  This is needed for the
//! following cases:
//! * '\\'
//! * '\['
//! * '\]'
//! * '\|'

mod markup_tag;
use self::markup_tag::Markup;

use std::rc::Rc;

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{Font, ResourceSet};
use sulis_core::ui::{FontRenderer, WidgetState};

pub struct MarkupRenderer {
    font: Rc<Font>,
    width: f32,
    draw_lists: Vec<DrawList>,
    bottom_y: f32,
    right_x: f32,
}

/// Struct for rendering text that is marked up with the simple
/// Markup language described in the `markup_tag` module documentation
impl MarkupRenderer {
    pub fn new(font: &Rc<Font>, width: i32) -> MarkupRenderer {
        let draw_lists = Vec::new();
        MarkupRenderer {
            font: Rc::clone(font),
            width: width as f32,
            draw_lists,
            bottom_y: 0.0,
            right_x: 0.0,
        }
    }

    pub fn text_bottom(&self) -> i32 {
        self.bottom_y as i32
    }

    pub fn text_right(&self) -> i32 {
        self.right_x.ceil() as i32
    }

    /// This sets up the drawing cache for this renderer.  it should be
    /// called when laying out the widget, once its position, size, and
    /// text are set.
    pub fn render_to_cache(&mut self, widget_state: &WidgetState) {
        let text = &widget_state.text;
        let defaults = &widget_state.text_params;

        let mut escaped = false;
        let mut in_markup_tag = false;
        let mut markup_stack: Vec<Markup> = Vec::new();
        let mut cur_markup = Markup::from_text_params(defaults, &self.font);
        let mut markup_buf = String::new();
        let mut word_buf = String::new();
        let mut word_width = 0;
        let pos_x = widget_state.inner_left() as f32;
        let pos_y = widget_state.inner_top() as f32 + cur_markup.scale / 2.0;
        let max_x = pos_x + self.width;
        let mut x = pos_x;
        let mut y = pos_y;

        for c in text.chars() {
            if escaped {
                word_buf.push(c);
                word_width += cur_markup.font.get_char_width(c);
                escaped = false;
            } else {
                match c {
                    '\\' => {
                        escaped = true;
                    }, '[' => {
                        let (x1, y1) = self.draw_current(&mut word_buf, &cur_markup,
                                                    x, y, pos_x, max_x, word_width);
                        x = x1;
                        y = y1;
                        word_width = 0;
                        in_markup_tag = true;
                    }, '|' => {
                        in_markup_tag = false;
                        markup_stack.push(cur_markup);
                        cur_markup = Markup::from_string(&markup_buf, &markup_stack.last().unwrap(),
                            widget_state);
                        markup_buf.clear();
                        if let Some(markup_x) = cur_markup.pos_x {
                            x = pos_x + markup_x;
                        }
                        if let Some(markup_y) = cur_markup.pos_y {
                            y = pos_y + markup_y;
                        }
                        if let Some(ref image) = cur_markup.image {
                            self.draw_sprite(&image, &cur_markup, x, y);
                        }
                    }, ']' => {
                        let (x1, y1) = self.draw_current(&mut word_buf, &cur_markup,
                                                    x, y, pos_x, max_x, word_width);
                        x = x1;
                        y = y1;
                        word_width = 0;

                        match markup_stack.pop() {
                            Some(markup) => cur_markup = markup,
                            None => warn!("Invalid ']' in markup"),
                        }
                    }, ' ' | '\n' => {
                        if !in_markup_tag {
                            word_buf.push(c);
                            let (x1, y1) = self.draw_current(&mut word_buf, &cur_markup,
                                                        x, y, pos_x, max_x, word_width);
                            x = x1;
                            y = y1;
                            word_width = 0;
                        }
                    }, _ => {
                        if in_markup_tag {
                            markup_buf.push(c);
                        } else {
                            word_buf.push(c);
                            word_width += cur_markup.font.get_char_width(c);
                        }
                    }
                }
            }
        }

        self.draw_current(&mut word_buf, &cur_markup, x, y, pos_x, max_x, word_width);

        if self.bottom_y < widget_state.inner_top() as f32 {
            self.bottom_y = widget_state.inner_top() as f32;
        }

        if self.right_x < widget_state.inner_left() as f32 {
            self.right_x = widget_state.inner_left() as f32;
        }
    }

    fn draw_current(&mut self, word_buf: &mut String, markup: &Markup, mut x: f32, mut y: f32,
                    start_x: f32, max_x: f32, word_width: u32) -> (f32, f32) {
        let factor = markup.font.base as f32 / markup.font.line_height as f32;

        if markup.ignore {
            word_buf.clear();
            return (x, y);
        }

        let width = word_width as f32 * markup.scale / markup.font.line_height as f32;
        if x + width > max_x {
            x = start_x;
            y += markup.scale * factor;
        }

        let mut quads = Vec::with_capacity(word_buf.len());
        for c in word_buf.chars() {
            match c {
                '\n' => {
                    x = start_x;
                    y += markup.scale * factor;
                }, _ => {
                    x = markup.add_quad_and_advance(&mut quads, c, x, y);

                    let bottom_y = y + (markup.scale - 1.0) * factor;
                    if bottom_y > self.bottom_y {
                        self.bottom_y = bottom_y;
                    }

                    if x > self.right_x {
                        self.right_x = x;
                    }
                }
            }
        }
        let mut draw_list = DrawList::from_font(&markup.font.id, quads);
        draw_list.set_color(markup.color);
        self.append_to_draw_lists(draw_list);

        word_buf.clear();
        (x, y)
    }

    fn draw_sprite(&mut self, image: &str, markup: &Markup, x: f32, y: f32) {
        if markup.ignore { return; }

        let sprite = match ResourceSet::get_sprite(image) {
            Err(_) => {
                warn!("Unable to find sprite '{}'", image);
                return;
            },
            Ok(sprite) => sprite,
        };

        let x_over_y = sprite.size.width as f32 / sprite.size.height as f32;
        let mut draw_list = DrawList::from_sprite_f32(&sprite, x, y,
                                                      markup.scale * x_over_y, markup.scale);
        draw_list.set_color(markup.color);
        self.append_to_draw_lists(draw_list);
    }

    fn append_to_draw_lists(&mut self, mut draw_list: DrawList) {
        let mut added = false;
        for list in self.draw_lists.iter_mut() {
            if draw_list.texture == list.texture && draw_list.color_filter == list.color_filter {
                list.append(&mut draw_list);
                added = true;
            }
        }
        if !added {
            self.draw_lists.push(draw_list);
        }
    }
}

impl FontRenderer for MarkupRenderer {
    fn render(&self, renderer: &mut GraphicsRenderer, _pos_x: f32, _pos_y: f32,
              _widget_state: &WidgetState) {
        for draw_list in self.draw_lists.iter() {
            renderer.draw(draw_list.clone());
        }
    }

    fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
}
