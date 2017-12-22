use std::slice::Iter;

use self::AnimationState::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Base,
    MouseOver,
    MouseDown,
    Active,
}

impl AnimationState {
    pub fn get_text(&self) -> &str {
        match *self {
            Base => "base",
            MouseOver => "mouseover",
            MouseDown => "mousedown",
            Active => "active",
        }
    }

    pub fn iter() -> Iter<'static, AnimationState> {
        static STATES: [AnimationState; 4] = [
            Base,
            MouseOver,
            MouseDown,
            Active
        ];
        STATES.iter()
    }

    pub fn find(text: &str) -> Option<AnimationState> {
        match text {
            "base" => Some(Base),
            "mouseover" => Some(MouseOver),
            "mousedown" => Some(MouseDown),
            "active" => Some(Active),
            _ => None,
        }
    }
}
