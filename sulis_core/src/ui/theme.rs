//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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
use std::collections::HashMap;

use serde_derive::{Deserialize};

use crate::resource::ResourceSet;
use crate::ui::color::Color;
use crate::ui::{WidgetState, Border, LayoutKind};
use crate::util::{Size, Point};

#[derive(Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

impl Default for HorizontalAlignment {
    fn default() -> Self { HorizontalAlignment::Center }
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl Default for VerticalAlignment {
    fn default() -> Self { VerticalAlignment::Center }
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum SizeRelative {
    Zero,
    Max,
    ChildMax,
    ChildSum,
    Custom,
}

impl Default for SizeRelative {
    fn default() -> Self {
        SizeRelative::Zero
    }
}

#[derive(Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum PositionRelative {
    Zero,
    Center,
    Max,
    Custom,
    Mouse,
}

impl Default for PositionRelative {
    fn default() -> Self { PositionRelative::Zero }
}

#[derive(Debug, Clone, Copy)]
pub struct Relative {
    pub x: PositionRelative,
    pub y: PositionRelative,
    pub width: SizeRelative,
    pub height: SizeRelative,
}

#[derive(Debug, Clone)]
pub struct TextParams {
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,

    pub color: Color,
    pub scale: f32,
    pub font: String,
}

impl Default for TextParams {
    fn default() -> Self {
        TextParams {
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
            color: Color::default(),
            scale: 1.0,
            font: "normal".to_string(),
        }
    }
}

impl Default for Relative {
    fn default() -> Self {
        Relative {
            x: PositionRelative::Zero,
            y: PositionRelative::Zero,
            width: SizeRelative::Zero,
            height: SizeRelative::Zero,
        }
    }
}

pub const DEFAULT_THEME_ID: &'static str = "default";

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Kind {
    Ref, // a reference to a widget that will be added in rust code, or
         // simply used as a building block for another theme item
    Label, // a widget showing static text - defined purely in the theme
    Container, // a widget holding other widgets - defined purely in the theme
}

impl Default for Kind {
    fn default() -> Self { Kind::Ref }
}

#[derive(Debug)]
pub struct Theme {
    pub id: String,
    pub kind: Kind,
    pub layout: LayoutKind,
    pub layout_spacing: Border,
    pub border: Border,
    pub size: Size,
    pub position: Point,
    pub relative: Relative,

    pub text: Option<String>,
    pub text_params: TextParams,
    pub background: Option<String>,
    pub foreground: Option<String>,

    pub custom: HashMap<String, String>,

    pub parent: Option<String>,
    pub children: Vec<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            id: DEFAULT_THEME_ID.to_string(),
            layout: LayoutKind::default(),
            layout_spacing: Border::default(),
            border: Border::default(),
            size: Size::default(),
            position: Point::default(),
            relative: Relative::default(),
            text: None,
            text_params: TextParams::default(),
            background: None,
            foreground: None,
            custom: HashMap::default(),
            children: Vec::default(),
            kind: Kind::default(),
            parent: None,
        }
    }
}

impl Theme {
    /// Sets the text for the `WidgetState` based on the defined theme text.
    /// References such as '#0#' are expanded to the corresponding text arg
    /// stored in the WidgetState.  See `WidgetState#add_text_arg` and
    /// `expand_text_args`
    pub fn apply_text(&self, state: &mut WidgetState) {
        let out = match self.text {
            None => String::new(),
            Some(ref text) => expand_text_args(text, state),
        };

        state.set_text_content(out);
    }

    /// Sets the background image for the `WidgetState`.  See `apply_foreground`
    pub fn apply_background(&self, state: &mut WidgetState) {
        let out = match self.background {
            None => return,
            Some(ref text) => expand_text_args(text, state),
        };
        if out.is_empty() { return; }

        match ResourceSet::image(&out) {
            None => warn!("Unable to find image for background '{}'", out),
            Some(image) => state.set_background(Some(image)),
        }
    }

    /// Sets the foreground image for the `WidgetState` based on the
    /// defined theme image.  References are expanded.  See `WidgetState#add_text_arg`
    /// and `expand_text_args`
    pub fn apply_foreground(&self, state: &mut WidgetState) {
        if self.foreground.is_some() {

            let out = match self.foreground {
                None => return,
                Some(ref text) => expand_text_args(text, state),
            };
            if out.is_empty() { return; }

            match ResourceSet::image(&out) {
                None => warn!("Unable to find image '{}'", out),
                Some(image) => state.set_foreground(Some(image)),
            }
        }
    }

    pub fn get_custom_or_default<T: Copy + FromStr>(&self, key: &str, default: T) -> T {
        match self.custom.get(key) {
            None => default,
            Some(ref value) => {
                match <T>::from_str(value) {
                    Err(_) => {
                        warn!("Unable to parse value {} from key {}", value, key);
                        default
                    },
                    Ok(value) => value
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ThemeSet {
    themes: HashMap<String, Rc<Theme>>,
}

impl ThemeSet {
    pub(crate) fn new(themes: HashMap<String, Rc<Theme>>) -> ThemeSet {
        ThemeSet {
            themes,
        }
    }

    pub fn default_theme(&self) -> &Rc<Theme> {
        &self.themes[DEFAULT_THEME_ID]
    }

    pub fn contains(&self, id: &str) -> bool {
        self.themes.contains_key(id)
    }

    pub fn get(&self, id: &str) -> &Rc<Theme> {
        match self.themes.get(id) {
            None => {
                warn!("Theme not found: {}", id);
                &self.themes[DEFAULT_THEME_ID]
            },
            Some(theme) => theme,
        }
    }

    pub fn compute_theme_id(&self, parent_id: &str, id: &str) -> String {
        match self.themes.get(parent_id) {
            None => (),
            Some(theme) => match self.compute_theme_id_recursive(theme, id) {
                None => (),
                Some(id) => return id,
            }
        }

        format!("{}.{}", parent_id, id)
    }

    fn compute_theme_id_recursive(&self, theme: &Rc<Theme>, id: &str) -> Option<String> {
        for child_id in theme.children.iter() {
            if child_id == id {
                return Some(format!("{}.{}", theme.id, id));
            }

            let child_theme = &self.themes.get(child_id).unwrap();
            match child_theme.kind {
                Kind::Container => (),
                _ => continue,
            }

            match self.compute_theme_id_recursive(child_theme, id) {
                None => (),
                Some(id) => return Some(id),
            }
        }

        None
    }
}

/// Expands all references to text args in the given string. Text args are
/// surrounded by `#` before and after.  A `##` produces just one `#` in the output.
/// For example, if the text arg `name` is set to `John Doe`, then the String
/// `Hello, #name# ##1` will be expanded to `Hello, John Doe #1`
pub fn expand_text_args(text: &str, state: &WidgetState) -> String {
    let mut out = String::new();
    let mut cur_arg = String::new();
    let mut arg_accum = false;
    for c in text.chars() {
        if arg_accum {
            if c.is_whitespace() {
            } else if c == '#' {
                if cur_arg.len() == 0 {
                    // ## code just gives a #
                    out.push(c);
                } else {
                    let text_arg = match state.get_text_arg(&cur_arg) {
                        None => {
                            // trace!("No text arg '{}' for text '{}'", cur_arg, text);
                            ""
                        },
                        Some(arg) => arg,
                    };
                    out.push_str(text_arg);
                }
                arg_accum = false;
                cur_arg.clear();
            } else {
                cur_arg.push(c);
            }
        } else if c == '#' {
            arg_accum = true;
        } else {
            out.push(c);
        }
    }

    if cur_arg.len() > 0 {
        let text_arg = match state.get_text_arg(&cur_arg) {
            None => {
                warn!("Non existant text arg '{}' in text '{}'", cur_arg, text);
                ""
            },
            Some(arg) => arg,
        };
        out.push_str(text_arg);
    }

    out
}

