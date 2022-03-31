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

use std::cell::RefCell;
use std::rc::Rc;

use crate::io::event::{ClickKind, Kind};
use crate::io::{keyboard_event::Key, Event};
use crate::ui::{Cursor, Widget};

pub struct InputAction {
    pub kind: InputActionKind,
    pub state: InputActionState,
}

#[derive(Copy, Clone, Debug)]
pub enum InputActionState {
    Started,
    Stopped,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum InputActionKind {
    ToggleConsole,
    ConsoleHistoryPrevious,
    ConsoleHistoryNext,
    ToggleInventory,
    ToggleCharacter,
    ToggleMap,
    ToggleJournal,
    ToggleFormation,
    Back,
    EndTurn,
    Rest,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    ZoomIn,
    ZoomOut,
    QuickSave,
    SelectAll,
    SwapWeapons,
    SelectPartyMember1,
    SelectPartyMember2,
    SelectPartyMember3,
    SelectPartyMember4,
    QuickItem1,
    QuickItem2,
    QuickItem3,
    QuickItem4,
    ActivateAbility1,
    ActivateAbility2,
    ActivateAbility3,
    ActivateAbility4,
    ActivateAbility5,
    ActivateAbility6,
    ActivateAbility7,
    ActivateAbility8,
    ActivateAbility9,
    ActivateAbility10,
    Exit,
    MouseMove(f32, f32),
    MouseButton(ClickKind),
    MouseScroll(i32),
    CharReceived(char),
    RawKey(Key),
}

impl std::cmp::Eq for InputActionKind {}

// input action hashmap should never include the variants with data
impl std::cmp::PartialEq for InputActionKind {
    fn eq(&self, other: &InputActionKind) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::hash::Hash for InputActionKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        // we don't care if different data leads to the same hash
    }
}

impl InputAction {
    pub fn mouse_move(x: f32, y: f32) -> InputAction {
        InputAction {
            kind: InputActionKind::MouseMove(x, y),
            state: InputActionState::Started,
        }
    }

    pub fn mouse_pressed(kind: ClickKind) -> InputAction {
        InputAction {
            kind: InputActionKind::MouseButton(kind),
            state: InputActionState::Started,
        }
    }

    pub fn mouse_released(kind: ClickKind) -> InputAction {
        InputAction {
            kind: InputActionKind::MouseButton(kind),
            state: InputActionState::Stopped,
        }
    }

    pub fn raw_key(key: Key) -> InputAction {
        InputAction {
            kind: InputActionKind::RawKey(key),
            state: InputActionState::Started,
        }
    }

    pub fn exit() -> InputAction {
        InputAction {
            kind: InputActionKind::Exit,
            state: InputActionState::Started,
        }
    }

    pub fn char_received(c: char) -> InputAction {
        InputAction {
            kind: InputActionKind::CharReceived(c),
            state: InputActionState::Started,
        }
    }

    pub fn mouse_scroll(amount: i32) -> InputAction {
        InputAction {
            kind: InputActionKind::MouseScroll(amount),
            state: InputActionState::Started,
        }
    }

    pub fn handle(self, root: &Rc<RefCell<Widget>>) {
        use InputActionKind::*;
        // don't spam tons of mouse move actions in the event logs
        match self.kind {
            MouseMove(_, _) => (),
            _ => debug!("Received action {:?}", self.kind),
        }

        match self.kind {
            MouseButton(kind) => {
                match self.state {
                    InputActionState::Started => Cursor::press(root, kind),
                    InputActionState::Stopped => Cursor::release(root, kind),
                }
            }
            MouseMove(x, y) => Cursor::move_to(root, x, y),
            MouseScroll(scroll) => {
                if scroll > 0 {
                    let event = Event::new(Kind::KeyPress(ZoomIn));
                    Widget::dispatch_event(root, event);
                } else {
                    let event = Event::new(Kind::KeyPress(ZoomOut));
                    Widget::dispatch_event(root, event);
                }
            }
            CharReceived(c) => {
                Widget::dispatch_event(root, Event::new(Kind::CharTyped(c)));
            }
            RawKey(key) => {
                Widget::dispatch_event(root, Event::new(Kind::RawKey(key)));
            }
            _ => {
                let kind = match self.state {
                    InputActionState::Started => Kind::KeyPress(self.kind),
                    InputActionState::Stopped => Kind::KeyRelease(self.kind),
                };

                let event = Event::new(kind);
                Widget::dispatch_event(root, event);
            }
        }
    }
}
