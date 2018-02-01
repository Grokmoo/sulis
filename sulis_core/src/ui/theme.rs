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

use std::f32;
use std::i32;
use std::str::FromStr;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::rc::Rc;
use std::collections::HashMap;

use resource::{BuilderType, ResourceSet};
use util::Point;
use ui::{Border, color, Color, LayoutKind, Size, WidgetState};

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum PositionRelative {
    Zero,
    Center,
    Cursor,
    Max,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum SizeRelative {
    Zero,
    Max,
    ChildMax,
    ChildSum,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum HorizontalTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum VerticalTextAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextParams {
    pub horizontal_alignment: HorizontalTextAlignment,
    pub vertical_alignment: VerticalTextAlignment,
    pub color: Color,
    pub scale: f32,
    pub font: String,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams::from(None)
    }
}

impl TextParams {
    fn from(builder: Option<TextParamsBuilder>) -> TextParams {
        let src = if builder.is_some() {
            builder.unwrap()
        } else {
            TextParamsBuilder::as_none()
        };

        TextParams {
            horizontal_alignment: src.horizontal_alignment.unwrap_or(HorizontalTextAlignment::Center),
            vertical_alignment: src.vertical_alignment.unwrap_or(VerticalTextAlignment::Center),
            color: src.color.unwrap_or(color::WHITE),
            scale: src.scale.unwrap_or(1.0),
            font: src.font.unwrap_or("default".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub layout: LayoutKind,
    pub layout_spacing: Border,
    pub text: Option<String>,
    pub text_params: TextParams,
    pub name: String,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub border: Border,
    pub preferred_size: Size,
    pub width_relative: SizeRelative,
    pub height_relative: SizeRelative,
    pub x_relative: PositionRelative,
    pub y_relative: PositionRelative,
    pub position: Point,
    pub children: HashMap<String, Rc<Theme>>,
    pub custom: HashMap<String, String>,
}

impl Theme {
    pub fn new(name: &str, builder: ThemeBuilder) -> Theme {
        let mut children: HashMap<String, Rc<Theme>> = HashMap::new();

        if let Some(builder_children) = builder.children {
            for (id, child) in builder_children {
                children.insert(id.to_string(), Rc::new(Theme::new(&id, child)));
            }
        }

        let x_relative = builder.x_relative.unwrap_or(PositionRelative::Zero);
        let y_relative = builder.y_relative.unwrap_or(PositionRelative::Zero);
        let width_relative = builder.width_relative.unwrap_or(SizeRelative::Zero);
        let height_relative = builder.height_relative.unwrap_or(SizeRelative::Zero);
        let border = builder.border.unwrap_or(Border::as_zero());
        let position = builder.position.unwrap_or(Point::as_zero());
        let preferred_size = builder.preferred_size.unwrap_or(Size::as_zero());
        let text_params = TextParams::from(builder.text_params);
        let layout = builder.layout.unwrap_or(LayoutKind::Normal);
        let layout_spacing = builder.layout_spacing.unwrap_or(Border::as_zero());
        let custom = builder.custom.unwrap_or(HashMap::new());

        Theme {
            layout,
            layout_spacing,
            name: name.to_string(),
            background: builder.background,
            foreground: builder.foreground,
            border,
            preferred_size,
            width_relative,
            height_relative,
            text: builder.text,
            position,
            x_relative,
            y_relative,
            children,
            text_params,
            custom,
        }
    }

    /// Sets the text for the `WidgetState` based on the defined theme text.
    /// References such as '#0#' are expanded to the corresponding text arg
    /// stored in the WidgetState.  See `WidgetState#add_text_arg` and
    /// `expand_text_args`
    pub fn apply_text(&self, state: &mut WidgetState) {
        let out = expand_text_args(&self.text, state);

        state.set_text_content(out);
    }

    /// Sets the foreground image for the `WidgetState` based on the
    /// defined theme image.  References are expanded.  See `WidgetState#add_text_arg`
    /// and `expand_text_args`
    pub fn apply_foreground(&self, state: &mut WidgetState) {
        if self.foreground.is_some() {
            let out = expand_text_args(&self.foreground, state);
            match ResourceSet::get_image(&out) {
                None => warn!("Unable to find image {}", out),
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

/// Expands all references to text args in the given string. Text args are
/// surrounded by `#` before and after.  A `##` produces just one `#` in the output.
/// For example, if the text arg `name` is set to `John Doe`, then the String
/// `Hello, #name# ##1` will be expanded to `Hello, John Doe #1`
pub fn expand_text_args(text: &Option<String>, state: &WidgetState) -> String {
    let text = match text {
        &None => return String::new(),
        &Some(ref text) => text,
    };

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
                            warn!("Non existant text arg '{}' in text '{}'", cur_arg, text);
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

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct TextParamsBuilder {
    horizontal_alignment: Option<HorizontalTextAlignment>,
    vertical_alignment: Option<VerticalTextAlignment>,
    color: Option<Color>,
    scale: Option<f32>,
    font: Option<String>,
}

impl TextParamsBuilder {
    fn as_none() -> TextParamsBuilder {
        TextParamsBuilder {
            horizontal_alignment: None,
            vertical_alignment: None,
            color: None,
            scale: None,
            font: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ThemeBuilder {
    background: Option<String>,
    foreground: Option<String>,
    border: Option<Border>,
    preferred_size: Option<Size>,
    text: Option<String>,
    text_params: Option<TextParamsBuilder>,
    layout: Option<LayoutKind>,
    layout_spacing: Option<Border>,
    position: Option<Point>,
    x_relative: Option<PositionRelative>,
    y_relative: Option<PositionRelative>,
    width_relative: Option<SizeRelative>,
    height_relative: Option<SizeRelative>,
    children: Option<HashMap<String, ThemeBuilder>>,
    include: Option<Vec<String>>,
    from: Option<String>,
    custom: Option<HashMap<String, String>>,
}

pub const MAX_THEME_DEPTH: i32 = 20;

impl ThemeBuilder {
    pub fn expand_references(&mut self) -> Result<(), Error> {
        if self.from.is_some() {
            warn!("Ignored 'from' key at root theme level.");
        }

        if self.children.is_none() {
            return Ok(());
        }

        // take a clone of the whole tree.  this wastes some
        // space and time, but makes the code to expand references
        // vastly simpler
        let builders_clone = self.clone();

        if let Some(ref mut children) = self.children {
            for (_id, child) in children {
                match child.expand_recursive(&builders_clone, 0) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
            }
        }

        Ok(())
    }

    fn expand_recursive(&mut self, builders: &ThemeBuilder,
                        depth: i32) -> Result<(), Error> {
        if depth >= MAX_THEME_DEPTH {
            warn!("Truncated theme expansion at max depth of {}", MAX_THEME_DEPTH);
            warn!("This is most likely caused by a circular 'from' reference.");
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Exceeded maximum theme depth."));
        }

        if self.from.is_some() {
            self.expand_self(builders);
        }

        if let Some(ref mut children) = self.children {
            for (_id, child) in children {
                match child.expand_recursive(&builders, depth + 1) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
            }
        }

        Ok(())
    }

    fn expand_self(&mut self, builders: &ThemeBuilder) {
        let from = self.from.as_ref().unwrap().to_string();
        let from_theme = builders.find_theme("", &from);

        if let Some(mut from_theme) = from_theme {
            if from_theme.from.is_some() {
                from_theme.expand_self(builders);
            }
            self.copy_from(from_theme, builders);
        } else {
            warn!("Unable to expand from reference to theme '{}'", from);
        }
        // mark as already expanded
        self.from = None;
    }

    fn copy_from(&mut self, other: ThemeBuilder, builders: &ThemeBuilder) {
        if self.background.is_none() { self.background = other.background; }
        if self.foreground.is_none() { self.foreground = other.foreground; }
        if self.border.is_none() { self.border = other.border; }
        if self.preferred_size.is_none() { self.preferred_size = other.preferred_size; }
        if self.text.is_none() { self.text = other.text; }
        if self.position.is_none() { self.position = other.position; }
        if self.layout.is_none() { self.layout = other.layout; }
        if self.layout_spacing.is_none() { self.layout_spacing = other.layout_spacing; }
        if self.x_relative.is_none() { self.x_relative = other.x_relative; }
        if self.y_relative.is_none() { self.y_relative = other.y_relative; }
        if self.width_relative.is_none() { self.width_relative = other.width_relative; }
        if self.height_relative.is_none() { self.height_relative = other.height_relative; }

        if let Some(other_custom) = other.custom {
            match self.custom {
                None => self.custom = Some(other_custom),
                Some(ref mut self_custom) => {
                    for (key, value) in other_custom {
                        if !self_custom.contains_key(&key) {
                            self_custom.insert(key, value);
                        }
                    }
                }
            }
        }

        if let Some(other_text_params) = other.text_params {
            match self.text_params {
                None => self.text_params = Some(other_text_params),
                Some(ref mut text_params) => {
                    if text_params.horizontal_alignment.is_none() {
                        text_params.horizontal_alignment = other_text_params.horizontal_alignment;
                    }
                    if text_params.vertical_alignment.is_none() {
                        text_params.vertical_alignment = other_text_params.vertical_alignment;
                    }
                    if text_params.color.is_none() { text_params.color = other_text_params.color; }
                    if text_params.scale.is_none() { text_params.scale = other_text_params.scale; }
                    if text_params.font.is_none() { text_params.font = other_text_params.font; }
                }
            }
        }

        // copy over only those children which aren't specified in this theme
        if let Some(mut other_children) = other.children {
            if self.children.is_none() {
                self.children = Some(HashMap::new());
            }

            // expand any refs in children before copying them over
            for (_, mut child) in other_children.iter_mut() {
                if child.from.is_some() {
                    child.expand_self(builders);
                }
            }

            for (id, child) in other_children {
                if !self.children.as_ref().unwrap().contains_key(&id) {
                    self.children.as_mut().unwrap().insert(id, child);
                } else {
                    let mut self_child = self.children.as_mut().unwrap().get_mut(&id);
                    let mut self_child_unwrapped = self_child.as_mut().unwrap();
                    self_child_unwrapped.copy_from(child, builders);
                }
            }
        }
    }

    fn find_theme(&self, cur_path: &str, id: &str) -> Option<ThemeBuilder> {
        if let Some(ref children) = self.children {
            for (child_id, child) in children {
                let child_path = format!("{}.{}", cur_path, child_id);
                trace!("Expanding theme references in {}", child_path);
                if child_path == id {
                    return Some(child.clone());
                }

                let result = child.find_theme(&child_path, id);

                if result.is_some() {
                    return result;
                }
            }
        }

        None
    }

    fn new(dir: &str, data: &str, builder_type: BuilderType) -> Result<ThemeBuilder, Error> {
        let mut theme = if builder_type == BuilderType::JSON {
            serde_json::from_str(data)?
        } else if builder_type == BuilderType::YAML {
            let resource: Result<ThemeBuilder, serde_yaml::Error> = serde_yaml::from_str(data);
            match resource {
                Ok(resource) => resource,
                Err(error) => return Err(Error::new(ErrorKind::InvalidData, format!("{}", error))),
            }
        } else {
            return Err(Error::new(ErrorKind::InvalidInput, "format not supported"))
        };

        if let None = theme.children {
            theme.children = Some(HashMap::new())
        }

        if let Some(ref includes) = theme.include {
            let theme_children = theme.children.as_mut().unwrap();

            for include_file in includes {
                let child_theme = match create_theme(dir, include_file) {
                    Ok(child_theme) => {
                        info!("Included theme '{}'", include_file);
                        child_theme
                    },
                    Err(e) => {
                        warn!("Unable to include theme '{}'", include_file);
                        warn!("{}", e);
                        continue;
                    }
                };

                if let Some(children) = child_theme.children {
                    for (id, child) in children {
                        theme_children.insert(id.to_string(), child);
                    }
                }
            }
        }

        Ok(theme)
    }
}

pub fn create_theme(dir: &str, filename: &str) -> Result<ThemeBuilder, Error> {
    let mut builder_type = BuilderType::JSON;
    let mut file = File::open(format!("{}{}.json", dir, filename));
    if file.is_err() {
        file = File::open(format!("{}{}.yml", dir, filename));
        builder_type = BuilderType::YAML;
    }

    if file.is_err() {
        return Err(Error::new(ErrorKind::NotFound,
            format!("Unable to locate '{}.json' or '{}.yml'", filename, filename)));
    }

    let mut file_data = String::new();
    file.unwrap().read_to_string(&mut file_data)?;
    let theme = ThemeBuilder::new(dir, &file_data, builder_type)?;

    Ok(theme)
}
