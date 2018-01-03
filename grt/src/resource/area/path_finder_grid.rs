use std::rc::Rc;
use std::fmt;

use resource::EntitySize;
use resource::Terrain;

pub struct PathFinderGrid {
    pub size: Rc<EntitySize>,
    pub passable: Vec<bool>,
    pub width: i32,
    pub height: i32,
}

impl fmt::Debug for PathFinderGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PathFinderGrid of size {}\n  ", self.size.size)?;
        for y in 0..self.height {
            for x in 0..self.width {
                if *self.passable.get((x + y * self.width) as usize).unwrap() {
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
    pub fn new(size: Rc<EntitySize>, terrain: &Terrain) -> PathFinderGrid {
        let width = terrain.width;
        let height = terrain.height;

        let mut passable = vec![false;(width * height) as usize];

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
                *passable.get_mut((x + y * width) as usize).unwrap() = is_passable;
            }
        }

        PathFinderGrid {
            size,
            passable,
            width,
            height,
        }
    }

    pub fn size(&self) -> i32 {
        self.size.size
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        *self.passable.get((x + y * self.width) as usize).unwrap()
    }

    pub fn is_passable_index(&self, index: i32) -> bool {
        *self.passable.get(index as usize).unwrap()
    }
}
