use std::rc::Rc;
use std::cell::RefCell;

use state::GameState;
use io::{Event, TextRenderer};
use ui::{Border, Size, WidgetState, WidgetKind};
use resource::Point;

pub struct Widget<'a> {
    pub state: WidgetState,
    pub kind: Rc<WidgetKind<'a> + 'a>,
    pub children: Vec<Rc<RefCell<Widget<'a>>>>,
    pub modal: Option<Rc<RefCell<Widget<'a>>>>,
}

impl<'a> Widget<'a> {
    fn new(kind: Rc<WidgetKind<'a> + 'a>, size: Size, position: Point,
           border: Border) -> Widget<'a> {
        let mut widget = Widget {
            state: WidgetState::new(size, position, border),
            kind: Rc::clone(&kind),
            children: Vec::new(),
            modal: None,
        };
        kind.on_add(&mut widget);

        widget
    }

    pub fn with_defaults(widget: Rc<WidgetKind<'a> + 'a>) -> Widget<'a> {
        Widget::new(widget, Size::as_zero(), Point::as_zero(), Border::as_zero())
    }

    pub fn with_size(widget: Rc<WidgetKind<'a> + 'a>,
                     size: Size) -> Widget<'a> {
        Widget::new(widget, size, Point::as_zero(), Border::as_zero())
    }

    pub fn with_position(widget: Rc<WidgetKind<'a> + 'a>, size: Size,
                         position: Point) -> Widget<'a> {
        Widget::new(widget, size, position, Border::as_zero())
    }

    pub fn with_border(widget: Rc<WidgetKind<'a> + 'a>, size: Size,
                       position: Point, border: Border) -> Widget<'a> {
        Widget::new(widget, size, position, border)
    }

    pub fn add_child(&mut self, widget: Widget<'a>) {
        self.add_child_private(Rc::new(RefCell::new(widget)), false);
    }

    pub fn add_child_rc(&mut self, widget: Rc<RefCell<Widget<'a>>>) {
        self.add_child_private(widget, false);
    }

    fn add_child_private(&mut self, widget: Rc<RefCell<Widget<'a>>>, modal: bool) {
        trace!("Adding {:?} to {:?}", widget.borrow().kind.get_name(),
            self.kind.get_name());
        if modal {
            self.modal = Some(Rc::clone(&widget));
        }

        self.children.push(widget);

    }

    pub fn dispatch_event(&mut self, state: &mut GameState, event: Event) -> bool {
        trace!("Dispatching event {:?} in {:?}", event, self.kind.get_name());

        if let Some(ref mut child) = self.modal {
            trace!("Found modal child");

            return child.borrow_mut().dispatch_event(state, event);
        }

        for child in self.children.iter_mut() {
            let mut child = child.borrow_mut();
            if child.state.in_bounds(event.mouse) {
                if !child.state.mouse_is_inside {
                    child.dispatch_event(state, Event::entered_from(&event));
                }

                if child.dispatch_event(state, event) {
                    return true;
                }
            } else if child.state.mouse_is_inside {
                child.dispatch_event(state, Event::exited_from(&event));
            }
        }

        let ref widget_kind = Rc::clone(&self.kind);
        use io::event::Kind::*;
        match event.kind {
            MouseClick(kind) =>
                widget_kind.on_mouse_click(state, self, kind, event.mouse),
            MouseMove { change: _change } =>
                widget_kind.on_mouse_move(state, self, event.mouse),
            MouseEnter =>
                widget_kind.on_mouse_enter(state, self, event.mouse),
            MouseExit =>
                widget_kind.on_mouse_exit(state, self, event.mouse),
            MouseScroll { scroll } =>
                widget_kind.on_mouse_scroll(state, self, scroll, event.mouse),
            KeyPress(action) =>
                widget_kind.on_key_press(state, self, action, event.mouse),
        }
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        if let Some(ref image) = self.state.background {
            image.fill_text_mode(renderer, self.state.animation_state.get_text(),
                &self.state.position, &self.state.size);
        }

        self.kind.draw_text_mode(renderer, self);

        for child in self.children.iter() {
            child.borrow().draw_text_mode(renderer);
        }
    }
}
