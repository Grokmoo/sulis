use ui::Border;

#[derive(Copy, Clone, Debug)]
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
    pub fn inner(&self, border: &Border) -> Size {
        Size {
            width: self.width - border.top - border.bottom,
            height: self.height - border.left - border.right,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }
}
