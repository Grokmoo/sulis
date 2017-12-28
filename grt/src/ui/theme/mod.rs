use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::rc::Rc;
use std::collections::HashMap;

use resource::BuilderType;
use util::Point;
use ui::{AnimationState, Border, Size, WidgetState};

use serde_json;
use serde_yaml;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum PositionRelative {
    Zero,
    Center,
    Cursor,
    Max,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SizeRelative {
    Zero,
    Max,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum HorizontalTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum VerticalTextAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum TextFormat {
    Normal,
    Bold,
    Italic,
    Underline,
}

#[derive(Debug)]
pub struct Theme {
    pub text: Option<String>,
    pub name: String,
    pub background: Option<String>,
    pub border: Border,
    pub preferred_size: Size,
    pub width_relative: SizeRelative,
    pub height_relative: SizeRelative,
    pub x_relative: PositionRelative,
    pub y_relative: PositionRelative,
    pub position: Point,
    pub children: HashMap<String, Rc<Theme>>,
    pub horizontal_text_alignment: HorizontalTextAlignment,
    pub vertical_text_alignment: VerticalTextAlignment,
    pub text_format: HashMap<AnimationState, TextFormat>,
}

impl Theme {
    pub fn new(name: &str, builder: ThemeBuilder) -> Theme {

        let mut children: HashMap<String, Rc<Theme>> = HashMap::new();

        if let Some(builder_children) = builder.children {
            for (id, child) in builder_children {
                children.insert(id.to_string(), Rc::new(Theme::new(&id, child)));
            }
        }

        let mut text_format: HashMap<AnimationState, TextFormat> = HashMap::new();
        if let Some(builder_text_format) = builder.text_format {
            for (state_str, format) in builder_text_format {
                match AnimationState::parse(&state_str) {
                    Ok(state) => { text_format.insert(state, format); }
                    Err(e) => {
                        warn!("The animation state '{}' could not be parsed \
                              for the text_format '{:?}'", state_str, format);
                        warn!("{}", e);
                    }
                };
            }
        }

        let x_relative = builder.x_relative.unwrap_or(PositionRelative::Zero);
        let y_relative = builder.y_relative.unwrap_or(PositionRelative::Zero);
        let width_relative = builder.width_relative.unwrap_or(SizeRelative::Zero);
        let height_relative = builder.height_relative.unwrap_or(SizeRelative::Zero);
        let border = builder.border.unwrap_or(Border::as_zero());
        let position = builder.position.unwrap_or(Point::as_zero());
        let preferred_size = builder.preferred_size.unwrap_or(Size::as_zero());
        let horizontal_text_alignment =
            builder.horizontal_text_alignment.unwrap_or(HorizontalTextAlignment::Center);
        let vertical_text_alignment =
            builder.vertical_text_alignment.unwrap_or(VerticalTextAlignment::Center);

        Theme {
            name: name.to_string(),
            background: builder.background,
            border,
            preferred_size,
            width_relative,
            height_relative,
            text: builder.text,
            position,
            x_relative,
            y_relative,
            children,
            horizontal_text_alignment,
            vertical_text_alignment,
            text_format,
        }
    }

    pub fn apply_text(&self, state: &mut WidgetState) {
        let text = match self.text {
            None => return,
            Some(ref text) => text,
        };

        let mut out = String::new();
        let mut param_next = false;
        for c in text.chars() {
            if param_next {
                if c == '#' {
                    // ## code just gives a #
                    out.push(c);
                } else {
                    let param_index = match c.to_digit(10) {
                        None => {
                            warn!("Invalid format string for text: '{}'", text);
                            return;
                        },
                        Some(index) => index,
                    };

                    let text_param = match state.get_text_param(param_index) {
                        None => {
                            warn!("Non existant text param '{}' in text '{}'", param_index, text);
                            return;
                        },
                        Some(param) => param,
                    };
                    out.push_str(text_param);
                }
                param_next = false;
            } else if c == '#' {
                param_next = true;
            } else {
                out.push(c);
            }
        }

        state.set_text_content(out);
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThemeBuilder {
    pub background: Option<String>,
    pub border: Option<Border>,
    pub preferred_size: Option<Size>,
    pub text: Option<String>,
    pub horizontal_text_alignment: Option<HorizontalTextAlignment>,
    pub vertical_text_alignment: Option<VerticalTextAlignment>,
    pub text_format: Option<HashMap<String, TextFormat>>,
    pub position: Option<Point>,
    pub x_relative: Option<PositionRelative>,
    pub y_relative: Option<PositionRelative>,
    pub width_relative: Option<SizeRelative>,
    pub height_relative: Option<SizeRelative>,
    pub children: Option<HashMap<String, ThemeBuilder>>,
    pub include: Option<Vec<String>>,
    pub from: Option<String>,
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
            let from = self.from.as_ref().unwrap().to_string();
            let from_theme = builders.find_theme("", &from);

            if let Some(from_theme) = from_theme {
                self.copy_from(from_theme);
            } else {
                warn!("Unable to expand from reference to theme '{}'", from);
            }
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

    fn copy_from(&mut self, other: ThemeBuilder) {
        if self.background.is_none() { self.background = other.background; }
        if self.border.is_none() { self.border = other.border; }
        if self.preferred_size.is_none() { self.preferred_size = other.preferred_size; }
        if self.text.is_none() { self.text = other.text; }
        if self.position.is_none() { self.position = other.position; }
        if self.x_relative.is_none() { self.x_relative = other.x_relative; }
        if self.y_relative.is_none() { self.y_relative = other.y_relative; }
        if self.width_relative.is_none() { self.width_relative = other.width_relative; }
        if self.height_relative.is_none() { self.height_relative = other.height_relative; }
        if self.horizontal_text_alignment.is_none() {
            self.horizontal_text_alignment = other.horizontal_text_alignment;
        }
        if self.vertical_text_alignment.is_none() {
            self.vertical_text_alignment = other.vertical_text_alignment;
        }

        // copy over only those states which aren't specified in this theme
        if let Some(other_text_format) = other.text_format {
            if self.text_format.is_none() {
                self.text_format = Some(HashMap::new());
            }

            for (state, format) in other_text_format {
                if !self.text_format.as_ref().unwrap().contains_key(&state) {
                    self.text_format.as_mut().unwrap().insert(state, format);
                }
            }
        }

        // copy over only those children which aren't specified in this theme
        if let Some(other_children) = other.children {
            if self.children.is_none() {
                self.children = Some(HashMap::new());
            }

            for (id, child) in other_children {
                if !self.children.as_ref().unwrap().contains_key(&id) {
                    self.children.as_mut().unwrap().insert(id, child);
                } else {
                    let mut self_child = self.children.as_mut().unwrap().get_mut(&id);
                    let mut self_child_unwrapped = self_child.as_mut().unwrap();
                    self_child_unwrapped.copy_from(child);
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
