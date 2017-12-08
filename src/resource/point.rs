use std::fmt;

use ui::Border;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn as_zero() -> Point {
        Point { x: 0, y: 0 }
    }

    pub fn new(x: i32, y: i32) -> Point {
        Point {x, y}
    }

    pub fn add(&self, x: i32, y: i32) -> Point {
        Point { x: &self.x + x, y: &self.y + y }
    }

    pub fn new_vec(data: Vec<(i32, i32)>) -> Vec<Point> {
        let mut vec = Vec::with_capacity(data.len());

        for (x, y) in data {
            vec.push(Point::new(x, y));
        }

        vec
    }

    pub fn inner(&self, border: &Border) -> Point {
        Point { x: self.x + border.left, y: self.y + border.top }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{},{}}}", self.x, self.y)
    }
}
