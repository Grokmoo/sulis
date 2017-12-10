use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};

use ui::{Border, Size, Widget};
use state::GameState;
use io::{MouseEvent, TextRenderer};

use resource::Point;

//// The base widget holder class.  Contains the common implementation across all
//// widgets, and holds an instance of 'Widget' which contains the specific behavior.
pub struct WidgetBase<'a> {
    pub position: Point,
    pub size: Size,
    pub border: Border,
    pub children: Vec<Rc<RefCell<WidgetBase<'a>>>>,
    widget: Rc<RefCell<Widget + 'a>>,
    mouse_is_inside: bool,
}

impl<'a> Debug for WidgetBase<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "Widget {} at {:?}", self.widget.borrow().get_name(),
            self.position)
    }
}

impl<'a> WidgetBase<'a> {
    pub fn with_defaults(widget: Rc<RefCell<Widget + 'a>>) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            widget,
            size: Size::as_zero(),
            position: Point::as_zero(),
            border: Border::as_zero(),
            children: Vec::new(),
            mouse_is_inside: false,
        }))
    }

    pub fn with_size(widget: Rc<RefCell<Widget + 'a>>,
                     size: Size) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            widget,
            size,
            position: Point::as_zero(),
            border: Border::as_zero(),
            children: Vec::new(),
            mouse_is_inside: false,
        }))
    }

    pub fn with_position(widget: Rc<RefCell<Widget + 'a>>, size: Size,
                         position: Point) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            widget,
            size,
            position,
            border: Border::as_zero(),
            children: Vec::new(),
            mouse_is_inside: false,
        }))
    }

    pub fn with_border(widget: Rc<RefCell<Widget + 'a>>, size: Size,
                       position: Point, border: Border) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            widget,
            size,
            position,
            border,
            children: Vec::new(),
            mouse_is_inside: false,
        }))
    }

    pub fn dispatch_event(&mut self, state: &mut GameState, event: MouseEvent) -> bool {
        trace!("Dispatching event {:?} in {:?}", event, self);
        for child in self.children.iter() {
            let mut child = child.borrow_mut();
            if child.in_bounds(event.x as i32, event.y as i32) {
                if !child.mouse_is_inside {
                    child.dispatch_event(state, MouseEvent::entered_from(&event));
                }

                if child.dispatch_event(state, event) {
                    return true;
                }
            } else if child.mouse_is_inside {
                child.dispatch_event(state, MouseEvent::exited_from(&event));
            }
        }

        let widget = Rc::clone(&self.widget);
        let widget = widget.borrow();
        use io::mouse_event::Kind::*;
        match event.kind {
            LeftClick => widget.on_left_click(self, state, event.x, event.y),
            MiddleClick => widget.on_middle_click(self, state, event.x, event.y),
            RightClick => widget.on_right_click(self, state, event.x, event.y),
            Move(_, _) => widget.on_mouse_moved(self, state, event.x, event.y),
            Entered => widget.on_mouse_entered(self, state, event.x, event.y),
            Exited => widget.on_mouse_exited(self, state, event.x, event.y),
            _ => false,
        }
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

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.size.in_bounds(x - self.position.x as i32, y - self.position.y as i32)
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.position = Point::new(
            x - (self.size.width - 1) / 2,
            y - (self.size.height - 1) / 2);
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.size = Size::new(width, height);
    }

    pub fn add_child(&mut self, widget: Rc<RefCell<WidgetBase<'a>>>) {
        trace!("Adding {:?} to {:?}", &widget.borrow(), self);
        self.children.push(widget);
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        self.widget.borrow_mut().draw_text_mode(renderer, &self);

        for child in self.children.iter() {
            let child = child.borrow();
            child.draw_text_mode(renderer);
        }
    }
}
