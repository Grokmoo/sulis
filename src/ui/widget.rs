use std::rc::Rc;
use std::cell::RefCell;

use uuid::Uuid;

use state::GameState;
use io::{Event, TextRenderer};
use ui::{Border, Size, WidgetState, WidgetKind};
use resource::{ResourceSet, Point};

pub struct Widget<'a> {
    pub state: WidgetState,
    pub kind: Rc<WidgetKind<'a> + 'a>,
    pub uuid: Uuid,
    children: Vec<Rc<RefCell<Widget<'a>>>>,
    modal_child: Option<Rc<RefCell<Widget<'a>>>>,
    parent: Option<Rc<RefCell<Widget<'a>>>>,
}

thread_local! {
    static MARKED_FOR_REMOVAL: RefCell<Vec<Uuid>> = RefCell::new(Vec::new());
}

impl<'a> Widget<'a> {
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

    pub fn mark_for_removal(&mut self) {
        trace!("Marked widget for removal '{}'", self.kind.get_name());
        MARKED_FOR_REMOVAL.with(|list| {
            list.borrow_mut().push(self.uuid);
        });
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Widget<'a>>>) {
        trace!("Adding {:?} to {:?}", child.borrow().kind.get_name(),
            self.kind.get_name());

        if child.borrow().state.is_modal {
            trace!("Adding child as modal widget.");
            self.modal_child = Some(Rc::clone(&child));
        }

        self.children.push(child);
    }

    pub fn add_children(&mut self, children: Vec<Rc<RefCell<Widget<'a>>>>) {
        for child in children.into_iter() {
            self.add_child(child);
        }
    }
}

impl<'a> Widget<'a> {
    fn new(kind: Rc<WidgetKind<'a> + 'a>, size: Size, position: Point,
           border: Border) -> Rc<RefCell<Widget<'a>>> {
        let widget = Widget {
            state: WidgetState::new(size, position, border),
            kind: Rc::clone(&kind),
            children: Vec::new(),
            modal_child: None,
            parent: None,
            uuid: Uuid::new_v4(),
        };

        let widget = Rc::new(RefCell::new(widget));
        let children = kind.on_add(&widget);
        widget.borrow_mut().add_children(children);

        widget
    }

    pub fn with_defaults(widget: Rc<WidgetKind<'a> + 'a>) -> Rc<RefCell<Widget<'a>>> {
        Widget::new(widget, Size::as_zero(), Point::as_zero(), Border::as_zero())
    }

    pub fn with_size(widget: Rc<WidgetKind<'a> + 'a>,
                     size: Size) -> Rc<RefCell<Widget<'a>>> {
        Widget::new(widget, size, Point::as_zero(), Border::as_zero())
    }

    pub fn with_position(widget: Rc<WidgetKind<'a> + 'a>, size: Size,
                         position: Point) -> Rc<RefCell<Widget<'a>>> {
        Widget::new(widget, size, position, Border::as_zero())
    }

    pub fn with_border(widget: Rc<WidgetKind<'a> + 'a>, size: Size,
                       position: Point, border: Border) -> Rc<RefCell<Widget<'a>>> {
        Widget::new(widget, size, position, border)
    }

    pub fn get_parent(widget: &Rc<RefCell<Widget<'a>>>) -> Rc<RefCell<Widget<'a>>> {
        Rc::clone(widget.borrow().parent.as_ref().unwrap())
    }

    pub fn set_background(widget: &mut Rc<RefCell<Widget<'a>>>, image: &str) {
        widget.borrow_mut().state.set_background(ResourceSet::get_image(image));
    }

    pub fn set_text(widget: &mut Rc<RefCell<Widget<'a>>>, text: &str) {
        widget.borrow_mut().state.set_text(text);
    }

    pub fn add_child_to(parent: &Rc<RefCell<Widget<'a>>>,
                         child: Rc<RefCell<Widget<'a>>>) {
        parent.borrow_mut().add_child(child);
    }

    pub fn add_children_to(parent: &Rc<RefCell<Widget<'a>>>,
                        children: Vec<Rc<RefCell<Widget<'a>>>>) {
        for child in children.into_iter() {
            Widget::add_child_to(parent, child);
        }
    }

    pub fn check_children(parent: &Rc<RefCell<Widget<'a>>>) {
        let mut remove_modal = false;
        if let Some(ref w) = parent.borrow().modal_child {
            MARKED_FOR_REMOVAL.with(|list| {
                if list.borrow().contains(&w.borrow().uuid) {
                    remove_modal = true;
                }
            });
        }

        if remove_modal {
            trace!("Removing modal widget.");
            parent.borrow_mut().modal_child = None;
        }

        parent.borrow_mut().children.retain(|w| {
            !MARKED_FOR_REMOVAL.with(|list| {
                list.borrow().contains(&w.borrow().uuid)
            })
        });

        let len = parent.borrow().children.len();
        for i in 0..len {
            {
                let parent = parent.borrow();
                if parent.children.get(i).unwrap().borrow().parent.is_some() {
                    continue;
                }
            }

            let child = Rc::clone(parent.borrow().children.get(i).unwrap());

            child.borrow_mut().parent = Some(Rc::clone(parent));

            Widget::check_children(&child);
        }
    }

    pub fn dispatch_event(widget: &Rc<RefCell<Widget<'a>>>,
                          state: &mut GameState, event: Event) -> bool {
        trace!("Dispatching event {:?} in {:?}", event,
               widget.borrow().kind.get_name());

        // precompute has modal so we don't have the widget borrowed
        // for the dispatch below
        let has_modal = widget.borrow().modal_child.is_some();
        if has_modal {
            trace!("Dispatching to modal child.");
            let child = Rc::clone(widget.borrow().modal_child.as_ref().unwrap());
            return Widget::dispatch_event(&child, state, event);
        }

        // iterate in this way using indices so we don't maintain any
        // borrows except for the active child widget - this will allow
        // the child to mutate any other widget in the tree
        let len = widget.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(widget.borrow().children.get(i).unwrap());

            if child.borrow().state.in_bounds(event.mouse) {
                if !child.borrow().state.mouse_is_inside {
                    Widget::dispatch_event(&child, state, Event::entered_from(&event));
                }

                if Widget::dispatch_event(&child, state, event) {
                    return true;
                }
            } else if child.borrow().state.mouse_is_inside {
                Widget::dispatch_event(&child, state, Event::exited_from(&event));
            }
        }

        let ref widget_kind = Rc::clone(&widget.borrow().kind);
        use io::event::Kind::*;
        match event.kind {
            MouseClick(kind) =>
                widget_kind.on_mouse_click(state, widget, kind, event.mouse),
            MouseMove { change: _change } =>
                widget_kind.on_mouse_move(state, widget, event.mouse),
            MouseEnter =>
                widget_kind.on_mouse_enter(state, widget, event.mouse),
            MouseExit =>
                widget_kind.on_mouse_exit(state, widget, event.mouse),
            MouseScroll { scroll } =>
                widget_kind.on_mouse_scroll(state, widget, scroll, event.mouse),
            KeyPress(action) =>
                widget_kind.on_key_press(state, widget, action, event.mouse),
        }
    }
}
