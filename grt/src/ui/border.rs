#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct Border {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Border {
    pub fn as_zero() -> Border {
        Border {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }

    pub fn as_uniform(border: i32) -> Border {
        Border {
            top: border,
            bottom: border,
            left: border,
            right: border,
        }
    }

    pub fn vertical(&self) -> i32 {
        self.top + self.bottom
    }

    pub fn horizontal(&self) -> i32 {
        self.right + self.left
    }
}
