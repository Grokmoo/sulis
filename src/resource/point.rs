use std::fmt;
use std::ops;

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
        Point { x, y }
    }

    pub fn from(other: &Point) -> Point {
        Point { x: other.x, y: other.y }
    }

    pub fn add(&self, x: i32, y: i32) -> Point {
        Point { x: &self.x + x, y: &self.y + y }
    }

    pub fn add_mut(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }

    pub fn add_x(&mut self, x: i32) {
        self.x += x;
    }

    pub fn add_y(&mut self, y: i32) {
        self.y += y;
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
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

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{},{}}}", self.x, self.y)
    }
}
