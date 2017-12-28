use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use io::{Event, TextRenderer};
use ui::{Size, Theme, WidgetState, WidgetKind};
use resource::ResourceSet;

pub struct Widget {
    pub state: WidgetState,
    pub kind: Rc<WidgetKind>,
    pub children: Vec<Rc<RefCell<Widget>>>,
    pub (in ui) theme: Option<Rc<Theme>>,
    pub theme_id: String,
    pub theme_subname: String,

    modal_child: Option<Rc<RefCell<Widget>>>,
    parent: Option<Rc<RefCell<Widget>>>,

    marked_for_removal: bool,
    marked_for_layout: bool,
    marked_for_readd: bool,
}

impl Widget {
    pub fn has_modal(&self) -> bool {
        self.modal_child.is_some()
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer, millis: u32) {
        if let Some(ref image) = self.state.background {
            image.fill_text_mode(renderer, &self.state.animation_state,
                &self.state.position, &self.state.size);
        }

        self.kind.draw_text_mode(renderer, self, millis);

        for child in self.children.iter() {
            child.borrow().draw_text_mode(renderer, millis);
        }
    }

    pub fn set_theme_name(&mut self, name: &str) {
        self.theme_subname = name.to_string();
    }

    pub fn mark_for_removal(&mut self) {
        trace!("Marked widget for removal '{}'", self.kind.get_name());
        self.marked_for_removal = true;
    }

    /// Causes this widget and all of its children to be layed out
    /// again on the next UI update.
    /// TODO if this is called in code during the layout process
    /// will create a loop where the widget is layed out every
    /// frame.  detect and prevent this
    pub fn invalidate_layout(&mut self) {
        self.marked_for_layout = true;
    }

    /// Causes this widget and all of its children to be removed and
    /// then the widget re-built on the next UI update.
    /// TODO loop potential, see `invalidate_layout`
    pub fn invalidate_children(&mut self) {
        self.marked_for_readd = true;
        self.marked_for_layout = true;
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Widget>>) {
        trace!("Adding {:?} to {:?}", child.borrow().kind.get_name(),
            self.kind.get_name());

        if child.borrow().state.is_modal {
            trace!("Adding child as modal widget.");
            self.modal_child = Some(Rc::clone(&child));
        }

        self.children.push(child);
    }

    pub fn add_children(&mut self, children: Vec<Rc<RefCell<Widget>>>) {
        for child in children.into_iter() {
            self.add_child(child);
        }
    }

    pub fn do_base_layout(&mut self) {
        self.do_self_layout();
        self.do_children_layout();
    }

    pub fn do_self_layout(&mut self) {
        let theme = match self.theme {
            None => return,
            Some(ref t) => t,
        };

        if let Some(ref bg) = theme.background {
            self.state.set_background(ResourceSet::get_image(&bg));
        }

        self.state.set_border(theme.border.clone());
        self.state.horizontal_text_alignment = theme.horizontal_text_alignment;
        self.state.vertical_text_alignment = theme.vertical_text_alignment;

        theme.apply_text(&mut self.state);
    }

    pub fn do_children_layout(&self) {
        for child in self.children.iter() {
            let theme = match child.borrow().theme {
                None => continue,
                Some(ref t) => Rc::clone(t),
            };

            let (w, h) = {
                use ui::theme::SizeRelative::*;
                (match theme.width_relative {
                    Zero => theme.preferred_size.width,
                    Max => self.state.inner_size.width + theme.preferred_size.width,
                },
                match theme.height_relative {
                    Zero => theme.preferred_size.height,
                    Max => self.state.inner_size.height + theme.preferred_size.height,
                })
            };

            let width = cmp::min(self.state.inner_size.width, w);
            let height = cmp::min(self.state.inner_size.height, h);
            child.borrow_mut().state.set_size(Size::new(width, height));

            use ui::theme::PositionRelative::*;
            let x = match theme.x_relative {
                Zero => self.state.inner_left(),
                Center => (self.state.inner_left() + self.state.inner_right() -
                           width) / 2,
                Max => self.state.inner_right() - width,
            };
            let y = match theme.y_relative {
                Zero => self.state.inner_top(),
                Center => (self.state.inner_top() + self.state.inner_bottom() -
                           height) / 2,
                Max => self.state.inner_bottom() - height,
            };

            child.borrow_mut().state.set_position(
                x + theme.position.x, y + theme.position.y);
        }
    }

    fn layout_widget(&mut self) {
        if self.marked_for_layout {
            trace!("Performing layout on widget {} with size {:?} at {:?}",
                   self.theme_id, self.state.size, self.state.position);
            let kind = Rc::clone(&self.kind);
            kind.layout(self);
            self.marked_for_layout = false;
        }

        let len = self.children.len();
        for i in 0..len {
            let child = Rc::clone(self.children.get(i).unwrap());
            child.borrow_mut().layout_widget();
        }
    }
}

impl Widget {
    fn new(kind: Rc<WidgetKind>, theme: &str) -> Rc<RefCell<Widget>> {
        let widget = Widget {
            state: WidgetState::new(),
            kind: Rc::clone(&kind),
            children: Vec::new(),
            modal_child: None,
            parent: None,
            marked_for_layout: true,
            theme: None,
            theme_id: String::new(),
            theme_subname: theme.to_string(),
            marked_for_removal: false,
            marked_for_readd: false,
        };

        let widget = Rc::new(RefCell::new(widget));
        let children = kind.on_add(&widget);
        widget.borrow_mut().add_children(children);

        widget
    }

    pub fn with_defaults(widget: Rc<WidgetKind>) -> Rc<RefCell<Widget>> {
        let name = widget.get_name().to_string();
        Widget::new(widget, &name)
    }

    pub fn with_theme(widget: Rc<WidgetKind>,
                      theme: &str) -> Rc<RefCell<Widget>> {
        Widget::new(widget, theme)
    }

    pub fn go_up_tree(widget: &Rc<RefCell<Widget>>,
                      levels: usize) -> Rc<RefCell<Widget>> {
        if levels == 0 {
            return Rc::clone(widget);
        }
        Widget::go_up_tree(&Widget::get_parent(widget), levels - 1)
    }

    pub fn get_parent(widget: &Rc<RefCell<Widget>>) -> Rc<RefCell<Widget>> {
        Rc::clone(widget.borrow().parent.as_ref().unwrap())
    }

    pub fn add_child_to(parent: &Rc<RefCell<Widget>>,
                         child: Rc<RefCell<Widget>>) {
        parent.borrow_mut().add_child(child);
        parent.borrow_mut().marked_for_layout = true;
    }

    pub fn add_children_to(parent: &Rc<RefCell<Widget>>,
                        children: Vec<Rc<RefCell<Widget>>>) {
        for child in children.into_iter() {
            Widget::add_child_to(parent, child);
        }
    }

    pub fn get_child_with_name(widget: &Rc<RefCell<Widget>>,
                               name: &str) -> Option<Rc<RefCell<Widget>>> {
        for child in widget.borrow().children.iter() {
            if child.borrow().kind.get_name() == name {
                return Some(Rc::clone(child));
            }
        }
        None
    }

    pub fn update(root: &Rc<RefCell<Widget>>) -> Result<(), Error> {
        Widget::check_readd(&root);
        Widget::check_children(&root)?;

        root.borrow_mut().layout_widget();

        Ok(())
    }

    pub fn check_readd(parent: &Rc<RefCell<Widget>>) {
        let len = parent.borrow().children.len();
        for i in 0..len {
            let child = Rc::clone(parent.borrow().children.get(i).unwrap());
            let readd = child.borrow().marked_for_readd;

            if readd {
                child.borrow_mut().children.clear();
                let kind = Rc::clone(&child.borrow().kind);
                child.borrow_mut().add_children(kind.on_add(&child));
                child.borrow_mut().marked_for_readd = false;
                parent.borrow_mut().marked_for_layout = true;
            } else {
                Widget::check_readd(&child);
            }
        }
    }

    pub fn check_children(parent: &Rc<RefCell<Widget>>) -> Result<(), Error> {
        let mut remove_modal = false;
        if let Some(ref w) = parent.borrow().modal_child {
            if w.borrow().marked_for_removal {
                remove_modal = true;
            }
        }

        if remove_modal {
            trace!("Removing modal widget.");
            parent.borrow_mut().modal_child = None;
        }

        parent.borrow_mut().children.retain(|w| !w.borrow().marked_for_removal);

        // set up theme
        if parent.borrow().theme.is_none() {
            let parent_parent = Widget::get_parent(parent);

            let parent_parent_theme = match parent_parent.borrow().theme {
                None => return Err(Error::new(ErrorKind::InvalidData,
                    format!("No theme exists for {}",
                            parent_parent.borrow().kind.get_name()))),
                Some(ref t) => Rc::clone(&t),
            };

            let mut parent = parent.borrow_mut();
            let parent_name = parent.theme_subname.clone();
            parent.theme_id = format!("{}.{}", &parent_parent.borrow().theme_id,
                parent_name);
            let parent_theme = parent_parent_theme.children.get(&parent_name);

            parent.theme = Some(match parent_theme {
                None => return Err(Error::new(ErrorKind::InvalidData,
                            format!("No theme exists for {}", parent.theme_id))),
                Some(ref t) => Rc::clone(&t),
            });

            trace!("Got theme for {:?}", parent.theme_id);
        }

        // set up parent references
        let len = parent.borrow().children.len();
        for i in 0..len {
            let setup_parent = {
                let children = &parent.borrow().children;
                let child_parent = &children.get(i).unwrap().borrow().parent;
                child_parent.is_none()
            };


            let child = Rc::clone(parent.borrow().children.get(i).unwrap());
            if setup_parent {
                child.borrow_mut().parent = Some(Rc::clone(parent));
            }

            Widget::check_children(&child)?;
        }

        Ok(())
    }

    pub fn dispatch_event(widget: &Rc<RefCell<Widget>>, event: Event) -> bool {
        trace!("Dispatching event {:?} in {:?}", event,
               widget.borrow().theme_id);

        let ref widget_kind = Rc::clone(&widget.borrow().kind);

        // precompute has modal so we don't have the widget borrowed
        // for the dispatch below
        let has_modal = widget.borrow().modal_child.is_some();
        if has_modal {
            trace!("Dispatching to modal child.");
            let child = Rc::clone(widget.borrow().modal_child.as_ref().unwrap());
            return Widget::dispatch_event(&child, event);
        }

        // iterate in this way using indices so we don't maintain any
        // borrows except for the active child widget - this will allow
        // the child to mutate any other widget in the tree
        let mut event_eaten = false;

        let len = widget.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(widget.borrow().children.get(i).unwrap());

            if child.borrow().state.in_bounds(event.mouse) {
                if !child.borrow().state.mouse_is_inside {
                    trace!("Dispatch mouse entered to '{}'", child.borrow().theme_id);
                    Widget::dispatch_event(&child, Event::entered_from(&event));
                }

                if !event_eaten && Widget::dispatch_event(&child, event) {
                    event_eaten = true;
                }
            } else if child.borrow().state.mouse_is_inside {
                trace!("Dispatch mouse exited to '{}'", child.borrow().theme_id);
                Widget::dispatch_event(&child, Event::exited_from(&event));
            }
        }

        if event_eaten { return true; }

        use io::event::Kind::*;
        match event.kind {
            MouseClick(kind) =>
                widget_kind.on_mouse_click(widget, kind, event.mouse),
            MouseMove { change: _change } =>
                widget_kind.on_mouse_move(widget, event.mouse),
            MouseEnter =>
                widget_kind.on_mouse_enter(widget, event.mouse),
            MouseExit =>
                widget_kind.on_mouse_exit(widget, event.mouse),
            MouseScroll { scroll } =>
                widget_kind.on_mouse_scroll(widget, scroll, event.mouse),
            KeyPress(action) =>
                widget_kind.on_key_press(widget, action, event.mouse),
        }
    }
}
