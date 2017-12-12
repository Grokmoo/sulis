use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};

use ui::{Border, Size, Widget, AnimationState};
use state::GameState;
use io::{Event, TextRenderer};

use resource::Point;
use resource::Image;

//// The base widget holder class.  Contains the common implementation across all
//// widgets, and holds an instance of 'Widget' which contains the specific behavior.
pub struct WidgetBase<'a> {
    pub position: Point,
    pub size: Size,
    pub border: Border,
    pub children: Vec<Rc<RefCell<WidgetBase<'a>>>>,
    widget: Rc<RefCell<Widget<'a> + 'a>>,
    mouse_is_inside: bool,
    background: Option<Rc<Image>>,
    animation_state: AnimationState,
}

impl<'a> Debug for WidgetBase<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "Widget {} at {:?}", self.widget.borrow().get_name(),
            self.position)
    }
}

impl<'a> WidgetBase<'a> {
    fn new(widget: Rc<RefCell<Widget<'a> + 'a>>, size: Size, position: Point,
               border: Border) -> Rc<RefCell<WidgetBase<'a>>> {

        let widget_base = Rc::new(RefCell::new(WidgetBase {
            widget: Rc::clone(&widget),
            size,
            position,
            border,
            children: Vec::new(),
            mouse_is_inside: false,
            background: None,
            animation_state: AnimationState::Base,
        }));

        widget.borrow_mut().set_parent(&widget_base);

        widget_base
    }

    pub fn with_defaults(widget: Rc<RefCell<Widget<'a> + 'a>>) ->
        Rc<RefCell<WidgetBase<'a>>> {
        WidgetBase::new(widget, Size::as_zero(), Point::as_zero(), Border::as_zero())
    }

    pub fn with_size(widget: Rc<RefCell<Widget<'a> + 'a>>,
                     size: Size) -> Rc<RefCell<WidgetBase<'a>>> {
        WidgetBase::new(widget, size, Point::as_zero(), Border::as_zero())
    }

    pub fn with_position(widget: Rc<RefCell<Widget<'a> + 'a>>, size: Size,
                         position: Point) -> Rc<RefCell<WidgetBase<'a>>> {
        WidgetBase::new(widget, size, position, Border::as_zero())
    }

    pub fn with_border(widget: Rc<RefCell<Widget<'a> + 'a>>, size: Size,
                       position: Point, border: Border) -> Rc<RefCell<WidgetBase<'a>>> {
        WidgetBase::new(widget, size, position, border)
    }

    pub fn dispatch_event(&mut self, state: &mut GameState, event: Event) -> bool {
        trace!("Dispatching event {:?} in {:?}", event, self);
        for child in self.children.iter() {
            let mut child = child.borrow_mut();
            if child.in_bounds(event.mouse) {
                if !child.mouse_is_inside {
                    child.dispatch_event(state, Event::entered_from(&event));
                }

                if child.dispatch_event(state, event) {
                    return true;
                }
            } else if child.mouse_is_inside {
                child.dispatch_event(state, Event::exited_from(&event));
            }
        }

        let widget = Rc::clone(&self.widget);
        let mut widget = widget.borrow_mut();
        use io::event::Kind::*;
        match event.kind {
            MouseClick(kind) => widget.on_mouse_click(state, kind, event.mouse),
            MouseMove { change: _change } => widget.on_mouse_move(state,
                                                                  event.mouse),
            MouseEnter => widget.on_mouse_enter(state, event.mouse),
            MouseExit => widget.on_mouse_exit(state, event.mouse),
            MouseScroll { scroll } => widget.on_mouse_scroll(state,
                                                          scroll, event.mouse),
            KeyPress(action) => widget.on_key_press(state, action, event.mouse),
        }
    }

    pub fn set_animation_state(&mut self, state: AnimationState) {
        self.animation_state = state;
    }

    pub fn set_background(&mut self, image: Option<Rc<Image>>) {
        self.background = image;
    }

    pub(super) fn set_mouse_inside(&mut self, is_inside: bool) {
        self.mouse_is_inside = is_inside;
    }

    pub fn inner_position(&self) -> Point {
        self.position.inner(&self.border)
    }

    pub fn inner_size(&self) -> Size {
        self.size.inner(&self.border)
    }

    pub fn in_bounds(&self, p: Point) -> bool {
        self.size.in_bounds(p.x - self.position.x as i32,
                            p.y - self.position.y as i32)
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.position = Point::new(
            x - (self.size.width - 1) / 2,
            y - (self.size.height - 1) / 2);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }

    pub fn add_child(&mut self, widget: Rc<RefCell<WidgetBase<'a>>>) {
        trace!("Adding {:?} to {:?}", &widget.borrow(), self);
        self.children.push(widget);
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        if let Some(ref image) = self.background {
            image.fill_text_mode(renderer, self.animation_state.get_text(),
                &self.position, &self.size);
        }

        self.widget.borrow_mut().draw_text_mode(renderer);

        for child in self.children.iter() {
            let child = child.borrow();
            child.draw_text_mode(renderer);
        }
    }
}
