use std::rc::Rc;

use ui::{AnimationState, Label, WidgetKind, WidgetState};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Label,
}

impl Button {
    pub fn new(text: &str) -> Rc<Button> {
        Rc::new(Button {
            label: Label {
                text: Some(text.to_string()),
            },
        })
    }
}

impl<'a> WidgetKind<'a> for Button {
    fn get_name(&self) -> &str {
        "Button"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        self.label.draw_text_mode(renderer);
    }

    fn on_mouse_enter(&self, _state: &mut GameState,
                      _mouse_pos: Point) -> bool {
        // self.base_ref.base_mut().set_mouse_inside(true);
        // self.base_ref.base_mut().set_animation_state(AnimationState::MouseOver);
        true
    }

    fn on_mouse_exit(&self, _state: &mut GameState,
                     _mouse_pos: Point) -> bool {
        // self.base_ref.base_mut().set_mouse_inside(false);
        // self.base_ref.base_mut().set_animation_state(AnimationState::Base);
        true
    }

    fn on_mouse_click(&self, state: &mut GameState,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        true
    }
}
