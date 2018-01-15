use io::InputAction;

use util::Point;

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub kind: Kind,
}

impl Event {
    pub fn new(kind: Kind) -> Event {
        Event {
            kind,
        }
    }

    pub fn entered_from(_event: &Event) -> Event {
        Event {
            kind: Kind::MouseEnter,
        }
    }

    pub fn exited_from(_event: &Event) -> Event {
        Event {
            kind: Kind::MouseExit,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    MousePress(ClickKind),
    MouseRelease(ClickKind),
    MouseMove { change: Point },
    MouseScroll { scroll: i32 },
    MouseEnter,
    MouseExit,
    KeyPress(InputAction),
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ClickKind {
    Left,
    Right,
    Middle,
}
