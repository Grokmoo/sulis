use std::fs::File;
use std::io::{Read, Error};
use std::rc::Rc;
use std::collections::HashMap;

use resource::Point;
use ui::{Border, Size};

use serde_json;

#[derive(Deserialize, Debug, Clone)]
pub enum PositionRelative {
    Zero,
    Center,
    Max,
}

#[derive(Deserialize, Debug, Clone)]
pub enum SizeRelative {
    Zero,
    Max,
}

#[derive(Debug)]
pub struct Theme {
    pub text: Option<String>,
    pub background: Option<String>,
    pub border: Border,
    pub preferred_size: Size,
    pub width_relative: SizeRelative,
    pub height_relative: SizeRelative,
    pub x_relative: PositionRelative,
    pub y_relative: PositionRelative,
    pub position: Point,
    pub children: HashMap<String, Rc<Theme>>,
}

impl Theme {
    pub fn new(builder: ThemeBuilder)
        -> Theme {

        let mut children: HashMap<String, Rc<Theme>> = HashMap::new();

        if let Some(builder_children) = builder.children {
            for (id, child) in builder_children {
                children.insert(id, Rc::new(Theme::new(child)));
            }
        }

        let x_relative = builder.x_relative.unwrap_or(PositionRelative::Zero);
        let y_relative = builder.y_relative.unwrap_or(PositionRelative::Zero);
        let width_relative = builder.width_relative.unwrap_or(SizeRelative::Zero);
        let height_relative = builder.height_relative.unwrap_or(SizeRelative::Zero);
        let border = builder.border.unwrap_or(Border::as_zero());
        let position = builder.position.unwrap_or(Point::as_zero());
        let preferred_size = builder.preferred_size.unwrap_or(Size::as_zero());

        Theme {
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
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThemeBuilder {
    pub background: Option<String>,
    pub border: Option<Border>,
    pub preferred_size: Option<Size>,
    pub text: Option<String>,
    pub position: Option<Point>,
    pub x_relative: Option<PositionRelative>,
    pub y_relative: Option<PositionRelative>,
    pub width_relative: Option<SizeRelative>,
    pub height_relative: Option<SizeRelative>,
    pub children: Option<HashMap<String, ThemeBuilder>>,
    pub include: Option<Vec<String>>,
    pub from: Option<String>,
}

impl ThemeBuilder {
    pub fn expand_references(&mut self) {
        if self.from.is_some() {
            warn!("Ignored 'from' key at root theme level.");
        }

        if self.children.is_none() {
            return;
        }

        // take a clone of the whole tree.  this wastes some
        // space and time, but makes the code to expand references
        // vastly simpler
        let builders_clone = self.clone();

        if let Some(ref mut children) = self.children {
            for (_id, child) in children {
                child.expand_recursive(&builders_clone);
            }
        }
    }

    fn expand_recursive(&mut self, builders: &ThemeBuilder) {
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
                child.expand_recursive(&builders);
            }
        }
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

    fn new(dir: &str, data: &str) -> Result<ThemeBuilder, Error> {
        let mut theme: ThemeBuilder = serde_json::from_str(data)?;

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
    let mut f = File::open(format!("{}{}", dir, filename))?;
    let mut file_data = String::new();
    f.read_to_string(&mut file_data)?;
    let theme = ThemeBuilder::new(dir, &file_data)?;

    Ok(theme)
}
