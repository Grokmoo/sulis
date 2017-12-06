pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl Cursor {
    pub fn move_by(&mut self, x: i32, y: i32) -> bool {
        let new_x = (self.x as i32) + x;
        let new_y = (self.y as i32) + y;

        if new_x < 0 || new_y < 0 {
            return false;
        }

        let new_x = new_x as usize;
        let new_y = new_y as usize;
        if new_x >= self.max_x || new_y >= self.max_y {
            return false;
        }

        self.x = new_x;
        self.y = new_y;

        true
    }
}
