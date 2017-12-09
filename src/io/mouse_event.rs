#[derive(Copy, Clone, Debug)]
pub struct MouseEvent {
    pub kind: Kind,
    pub x: i32,
    pub y: i32
}

impl MouseEvent {
    pub fn new(kind: Kind, mouse_x: i32, mouse_y: i32) -> MouseEvent {
        MouseEvent {
            kind,
            x: mouse_x,
            y: mouse_y,
        }
    }

    pub fn entered_from(event: &MouseEvent) -> MouseEvent {
        MouseEvent {
            kind: Kind::Entered,
            x: event.x,
            y: event.y,
        }
    }

    pub fn exited_from(event: &MouseEvent) -> MouseEvent {
        MouseEvent {
            kind: Kind::Exited,
            x: event.x,
            y: event.y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    LeftClick,
    RightClick,
    MiddleClick,
    Move(i32, i32),
    Scroll(i32),
    Entered,
    Exited,
}
