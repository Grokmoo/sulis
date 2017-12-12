use std::slice::Iter;

use self::AnimationState::*;

#[derive(Debug, PartialEq, Eq)]
pub enum AnimationState {
    Base,
    MouseOver,
}

impl AnimationState {
    pub fn get_text(&self) -> &str {
        match *self {
            Base => "base",
            MouseOver => "mouseover",
        }
    }

    pub fn iter() -> Iter<'static, AnimationState> {
        static STATES: [AnimationState; 2] = [ Base, MouseOver ];
        STATES.iter()
    }

    pub fn find(text: &str) -> Option<AnimationState> {
        match text {
            "base" => Some(Base),
            "mouseover" => Some(MouseOver),
            _ => None,
        }
    }
}
