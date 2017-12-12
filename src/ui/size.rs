use std::ops;

use ui::Border;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn as_zero() -> Size {
        Size { width: 0, height: 0 }
    }

    pub fn new(width: i32, height: i32) -> Size {
        Size {
            width,
            height,
        }
    }

    pub fn from(other: &Size) -> Size {
        Size {
            width: other.width,
            height: other.height,
        }
    }

    pub fn inner(&self, border: &Border) -> Size {
        Size {
            width: self.width - border.top - border.bottom,
            height: self.height - border.left - border.right,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn product(&self) -> i32 {
        self.width * self.height
    }

    pub fn add_mut(&mut self, width: i32, height: i32) {
        self.width += width;
        self.height += height;
    }

    pub fn set(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_width(&mut self, width: i32) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: i32) {
        self.height = height;
    }
}

impl ops::Add<Size> for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Size {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl ops::Sub<Size> for Size {
    type Output = Size;

    fn sub(self, rhs: Size) -> Size {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
