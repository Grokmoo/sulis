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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

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
