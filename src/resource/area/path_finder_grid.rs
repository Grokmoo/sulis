use std::rc::Rc;
use std::fmt;

use resource::Size;
use resource::Terrain;

pub struct PathFinderGrid {
    pub size: Rc<Size>,
    pub passable: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl fmt::Debug for PathFinderGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PathFinderGrid of size {}\n  ", self.size.size)?;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.passable.get(x + y * self.width).unwrap() {
                    write!(f, ".")?;
                } else {
                    write!(f, "X")?;
                }
            }
            write!(f, "\n  ")?;
        }
        write!(f, "\n")
    }
}

impl PathFinderGrid {
    pub fn new(size: Rc<Size>, terrain: &Terrain) -> PathFinderGrid {
        let width = terrain.width;
        let height = terrain.height;

        let mut passable = vec![false;width * height];

        for y in 0..height {
            for x in 0..width {
                let mut is_passable = true;
                for p in size.points(x, y) {
                    if p.x >= width || p.y >= height {
                        is_passable = false;
                        break;
                    }
                    if !terrain.is_passable(p.x, p.y) {
                        is_passable = false;
                        break;
                    }
                }
                *passable.get_mut(x + y * width).unwrap() = is_passable;
            }
        }

        PathFinderGrid {
            size,
            passable,
            width,
            height,
        }
    }

    pub fn size(&self) -> usize {
        self.size.size
    }

    pub fn is_passable(&self, x: usize, y: usize) -> bool {
        *self.passable.get(x + y * self.width).unwrap()
    }

    pub fn is_passable_index(&self, index: usize) -> bool {
        *self.passable.get(index).unwrap()
    }
}
