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

use crate::io::InputActionState;

#[derive(Copy, Clone, Debug)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: InputActionState,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum Key {
    KeyUnknown,

    KeyEscape,
    KeyBackspace,
    KeyTab,
    KeySpace,
    KeyEnter,
    KeyHome,
    KeyEnd,
    KeyInsert,
    KeyDelete,
    KeyPageUp,
    KeyPageDown,
    KeyGrave,
    KeyMinus,
    KeyEquals,
    KeyLeftBracket,
    KeyRightBracket,
    KeySemicolon,
    KeySingleQuote,
    KeyComma,
    KeyPeriod,
    KeySlash,
    KeyBackslash,

    KeyUp,
    KeyDown,
    KeyLeft,
    KeyRight,

    KeyF1,
    KeyF2,
    KeyF3,
    KeyF4,
    KeyF5,
    KeyF6,
    KeyF7,
    KeyF8,
    KeyF9,
    KeyF10,
    KeyF11,
    KeyF12,

    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
}

impl Key {
    pub fn short_name(self) -> String {
        let long_name = format!("{self:?}");

        long_name[3..].to_string()
    }
}
