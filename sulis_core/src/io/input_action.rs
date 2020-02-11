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

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum InputAction {
    ToggleConsole,
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
    MouseDown(ClickKind),
    MouseUp(ClickKind),
    MouseScroll(i32),
    CharReceived(char),
    RawKey(Key),
}

impl std::cmp::Eq for InputAction {}

// input action hashmap should never include the variants with data
impl std::cmp::PartialEq for InputAction {
    fn eq(&self, other: &InputAction) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::hash::Hash for InputAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        // we don't care if different data leads to the same hash
    }
}

impl InputAction {
    pub fn handle_action(action: InputAction, root: &Rc<RefCell<Widget>>) {
        // don't spam tons of mouse move actions in the event logs
        match action {
            MouseMove(_, _) => (),
            _ => debug!("Received action {:?}", action),
        }

        use crate::io::InputAction::*;
        match action {
            MouseMove(x, y) => Cursor::move_to(root, x, y),
            MouseDown(kind) => Cursor::press(root, kind),
            MouseUp(kind) => Cursor::release(root, kind),
            MouseScroll(scroll) => {
                if scroll > 0 {
                    InputAction::fire_action(ZoomIn, root);
                } else {
                    InputAction::fire_action(ZoomOut, root);
                }
            }
            CharReceived(c) => {
                Widget::dispatch_event(root, Event::new(Kind::CharTyped(c)));
            }
            RawKey(key) => {
                Widget::dispatch_event(root, Event::new(Kind::RawKey(key)));
            }
            _ => InputAction::fire_action(action, root),
        }
    }

    fn fire_action(action: InputAction, root: &Rc<RefCell<Widget>>) {
        debug!("Firing action {:?}", action);

        let event = Event::new(Kind::KeyPress(action));
        Widget::dispatch_event(&root, event);
    }
}
