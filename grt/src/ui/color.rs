#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

fn get_component(text: &str, max: f32) -> f32 {
    let component = i32::from_str_radix(&text, 16);
    match component {
        Err(_) => {
            warn!("Unable to parse color component from '{}'", text);
            1.0
        },
        Ok(c) => {
            c as f32 / max
        }
    }
}

impl Color {
    pub fn from_string(text: &str) -> Color {
        if text.len() == 3 || text.len() == 4 {
            let r = get_component(&text[0..1], 16.0);
            let g = get_component(&text[1..2], 16.0);
            let b = get_component(&text[2..3], 16.0);
            let a = if text.len() == 4 {
                get_component(&text[3..4], 16.0)
            } else {
                1.0
            };

            Color { r, g, b, a }
        } else if text.len() == 6 || text.len() == 8 {
            let r = get_component(&text[0..2], 255.0);
            let g = get_component(&text[2..4], 255.0);
            let b = get_component(&text[4..6], 255.0);
            let a = if text.len() == 8 {
                get_component(&text[6..8], 255.0)
            } else {
                1.0
            };

            Color { r, g, b, a }
        } else {
            warn!("Unable to parse color from string '{}'", text);
            WHITE
        }
    }
}

pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
pub const PURPLE: Color = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
pub const CYAN: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
