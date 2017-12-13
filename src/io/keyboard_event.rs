#[derive(Copy, Clone, Debug)]
pub struct KeyboardEvent {
    pub key: Key,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
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

    KeyUp, KeyDown, KeyLeft, KeyRight,

    KeyF1, KeyF2, KeyF3, KeyF4, KeyF5, KeyF6,
    KeyF7, KeyF8, KeyF9, KeyF10, KeyF11, KeyF12,

    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM,
    KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,

    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
}

