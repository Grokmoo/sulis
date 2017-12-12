use std::rc::Rc;
use std::cell::{RefCell, RefMut};

use ui::{AnimationState, BaseRef, Label, Widget, WidgetBase};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button<'a> {
    label: Label<'a>,
    callback: Option<Box<FnMut(RefMut<WidgetBase>, &mut GameState)>>,
    base_ref: BaseRef<'a>,
}

impl<'a> Button<'a> {
    pub fn new(text: &str) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label: Label {
                text: Some(text.to_string()),
                base_ref: BaseRef::new(),
            },
            callback: None,
            base_ref: BaseRef::new(),
        }))
    }

    pub fn with_callback(text: &str,
        callback: Box<FnMut(RefMut<WidgetBase>, &mut GameState)>) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label: Label {
                text: Some(text.to_string()),
                base_ref: BaseRef::new(),
            },
            callback: Some(callback),
            base_ref: BaseRef::new(),
        }))
    }

    pub fn set_callback(&mut self, callback: Box<FnMut(RefMut<WidgetBase>, &mut GameState)>) {
        self.callback = Some(callback);
    }
}

impl<'a> Widget<'a> for Button<'a> {
    fn set_parent(&mut self, parent: &Rc<RefCell<WidgetBase<'a>>>) {
        self.base_ref.set_base(parent);
    }

    fn get_name(&self) -> &str {
        "Button"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        self.label.draw_text_mode(renderer);
    }

    fn on_mouse_enter(&mut self, _state: &mut GameState,
                      _mouse_pos: Point) -> bool {
        self.base_ref.base_mut().set_mouse_inside(true);
        self.base_ref.base_mut().set_animation_state(AnimationState::MouseOver);
        true
    }

    fn on_mouse_exit(&mut self, _state: &mut GameState,
                     _mouse_pos: Point) -> bool {
        self.base_ref.base_mut().set_mouse_inside(false);
        self.base_ref.base_mut().set_animation_state(AnimationState::Base);
        true
    }

    fn on_mouse_click(&mut self, state: &mut GameState,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        if let Some(ref mut cb) = self.callback {
            cb(self.base_ref.base_mut(), state);
        }
        true
    }
}
