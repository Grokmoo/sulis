use io::InputAction;

use util::Point;

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub kind: Kind,
    pub mouse: Point
}

impl Event {
    pub fn new(kind: Kind, mouse_x: i32, mouse_y: i32) -> Event {
        Event {
            kind,
            mouse: Point::new(mouse_x, mouse_y),
        }
    }

    pub fn entered_from(event: &Event) -> Event {
        Event {
            kind: Kind::MouseEnter,
            mouse: event.mouse,
        }
    }

    pub fn exited_from(event: &Event) -> Event {
        Event {
            kind: Kind::MouseExit,
            mouse: event.mouse,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    MouseClick(ClickKind),
    MouseMove { change: Point },
    MouseScroll { scroll: i32 },
    MouseEnter,
    MouseExit,
    KeyPress(InputAction),
}

#[derive(Copy, Clone, Debug)]
pub enum ClickKind {
    Left,
    Right,
    Middle,
}
