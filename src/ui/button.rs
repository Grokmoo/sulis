use std::rc::Rc;
use std::cell::RefCell;

use ui::{AnimationState, Label, Widget, WidgetKind};
use io::{event, TextRenderer};
use state::GameState;
use resource::Point;

pub struct Button {
    label: Rc<Label>,
    callback: Box<Fn(&Rc<RefCell<Widget>>, &mut GameState)>,
}

impl Button {
    pub fn new(callback: Box<Fn(&Rc<RefCell<Widget>>, &mut GameState)>) -> Rc<Button> {
        Rc::new(Button {
            label: Label::new(),
            callback
        })
    }
}

impl<'a> WidgetKind<'a> for Button {
    fn get_name(&self) -> &str {
        "Button"
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer, widget: &Widget<'a>) {
        self.super_draw_text_mode(widget);

        self.label.draw_text_mode(renderer, widget);
    }

    fn on_mouse_enter(&self, _state: &mut GameState, widget: &Rc<RefCell<Widget<'a>>>,
                      _mouse_pos: Point) -> bool {
        self.super_on_mouse_enter(widget);
        widget.borrow_mut().state.set_animation_state(AnimationState::MouseOver);
        true
    }

    fn on_mouse_exit(&self, _state: &mut GameState, widget: &Rc<RefCell<Widget<'a>>>,
                     _mouse_pos: Point) -> bool {
        self.super_on_mouse_exit(widget);
        widget.borrow_mut().state.set_animation_state(AnimationState::Base);
        true
    }

    fn on_mouse_click(&self, state: &mut GameState, widget: &Rc<RefCell<Widget<'a>>>,
                      _kind: event::ClickKind, _mouse_pos: Point) -> bool {
        (self.callback)(widget, state);
        true
    }
}
