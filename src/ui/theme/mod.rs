use std::io::Error;
use std::rc::Rc;
use std::collections::HashMap;

use resource::{Point, ResourceBuilder};
use ui::{Border, Size};

use serde_json;

#[derive(Deserialize, Debug)]
pub enum PositionRelative {
    Zero,
    Center,
    Max,
}

#[derive(Deserialize, Debug)]
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
    pub fn new(builder: ThemeBuilder) -> Theme {
        let mut children: HashMap<String, Rc<Theme>> = HashMap::new();

        if builder.children.is_some() {
            for (id, child) in builder.children.unwrap() {
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

#[derive(Deserialize, Debug)]
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
}

impl ResourceBuilder for ThemeBuilder {
    fn owned_id(&self) -> String {
        "Theme".to_string()
    }

    fn new(data: &str) -> Result<ThemeBuilder, Error> {
        let theme: ThemeBuilder = serde_json::from_str(data)?;

        Ok(theme)
    }
}
