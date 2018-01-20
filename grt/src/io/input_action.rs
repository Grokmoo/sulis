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

use std::rc::Rc;
use std::cell::RefCell;

use ui::{Cursor, Widget};
use io::event::{ClickKind, Kind};
use io::Event;

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub enum InputAction {
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    ClickCursor,
    RightClickCursor,
    ToggleInventory,
    ToggleCharacter,
    Exit,
    MouseMove(f32, f32),
    MouseDown(ClickKind),
    MouseUp(ClickKind),
}

impl InputAction {
    pub fn handle_action(action: Option<InputAction>, root: Rc<RefCell<Widget>>) {
        let action = match action {
            None => return,
            Some(action) => action
        };

        // don't spam tons of mouse move actions in the event logs
        match action {
            MouseMove(_, _) => (),
            _ => debug!("Received action {:?}", action),
        }

        Widget::remove_mouse_over(&root);
        use io::InputAction::*;
        match action {
            MoveCursorUp => Cursor::move_by(&root, 0.0, -1.0),
            MoveCursorDown => Cursor::move_by(&root, 0.0, 1.0),
            MoveCursorLeft => Cursor::move_by(&root, -1.0, 0.0),
            MoveCursorRight => Cursor::move_by(&root, 1.0, 0.0),
            ClickCursor => {
                Cursor::press(&root, ClickKind::Left);
                Cursor::release(&root, ClickKind::Left)
            }
            RightClickCursor => {
                Cursor::press(&root, ClickKind::Right);
                Cursor::release(&root, ClickKind::Right)
            },
            MouseMove(x, y) => Cursor::move_to(&root, x, y),
            MouseDown(kind) => Cursor::press(&root, kind),
            MouseUp(kind) => Cursor::release(&root, kind),
            _ => InputAction::fire_action(action, root),
        }
    }

    fn fire_action(action: InputAction, root: Rc<RefCell<Widget>>) {
        debug!("Firing action {:?}", action);

        let event = Event::new(Kind::KeyPress(action));
        Widget::dispatch_event(&root, event);
    }
}
