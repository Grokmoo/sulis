//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use crate::io::{keyboard_event::Key, InputAction};

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub kind: Kind,
}

impl Event {
    pub fn new(kind: Kind) -> Event {
        Event { kind }
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
    MouseMove {
        delta_x: f32,
        delta_y: f32,
    },
    MouseDrag {
        button: ClickKind,
        delta_x: f32,
        delta_y: f32,
    },
    MouseScroll {
        scroll: i32,
    },
    MouseEnter,
    MouseExit,
    CharTyped(char),
    KeyPress(InputAction),
    RawKey(Key),
}

#[derive(Eq, Hash, Deserialize, Serialize, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum ClickKind {
    Left,
    Right,
    Middle,
}
