use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use uuid::Uuid;

use state::GameState;
use io::{Event, TextRenderer};
use ui::{Size, Theme, WidgetState, WidgetKind};
use resource::ResourceSet;

pub struct Widget<'a> {
    pub state: WidgetState,
    pub kind: Rc<WidgetKind<'a> + 'a>,
    pub uuid: Uuid,
    pub children: Vec<Rc<RefCell<Widget<'a>>>>,
    modal_child: Option<Rc<RefCell<Widget<'a>>>>,
    parent: Option<Rc<RefCell<Widget<'a>>>>,
    needs_layout: bool,
    pub (in ui) theme: Option<Rc<Theme>>,
    theme_id: String,
    theme_subname: String,
}

thread_local! {
    static MARKED_FOR_REMOVAL: RefCell<Vec<Uuid>> = RefCell::new(Vec::new());
}

impl<'a> Widget<'a> {
    pub fn has_modal(&self) -> bool {
        self.modal_child.is_some()
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

    pub fn set_theme_name(&mut self, name: &str) {
        self.theme_subname = name.to_string();
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

        if let Some(ref text) = theme.text {
            self.state.set_text(text);
        }
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
        if self.needs_layout {
            trace!("Performing layout on widget {} with size {:?} at {:?}",
                   self.theme_id, self.state.size, self.state.position);
            let kind = Rc::clone(&self.kind);
            kind.layout(self);
            self.needs_layout = false;
        }

        let len = self.children.len();
        for i in 0..len {
            let child = Rc::clone(self.children.get(i).unwrap());
            child.borrow_mut().layout_widget();
        }
    }
}

impl<'a> Widget<'a> {
    fn new(kind: Rc<WidgetKind<'a> + 'a>, theme: &str) -> Rc<RefCell<Widget<'a>>> {
        let widget = Widget {
            state: WidgetState::new(),
            kind: Rc::clone(&kind),
            children: Vec::new(),
            modal_child: None,
            parent: None,
            uuid: Uuid::new_v4(),
            needs_layout: true,
            theme: None,
            theme_id: String::new(),
            theme_subname: theme.to_string(),
        };

        let widget = Rc::new(RefCell::new(widget));
        let children = kind.on_add(&widget);
        widget.borrow_mut().add_children(children);

        widget
    }

    pub fn with_defaults(widget: Rc<WidgetKind<'a> + 'a>) -> Rc<RefCell<Widget<'a>>> {
        let name = widget.get_name().to_string();
        Widget::new(widget, &name)
    }

    pub fn with_theme(widget: Rc<WidgetKind<'a> + 'a>,
                      theme: &str) -> Rc<RefCell<Widget<'a>>> {
        Widget::new(widget, theme)
    }

    pub fn get_parent(widget: &Rc<RefCell<Widget<'a>>>) -> Rc<RefCell<Widget<'a>>> {
        Rc::clone(widget.borrow().parent.as_ref().unwrap())
    }

    pub fn add_child_to(parent: &Rc<RefCell<Widget<'a>>>,
                         child: Rc<RefCell<Widget<'a>>>) {
        parent.borrow_mut().add_child(child);
        parent.borrow_mut().needs_layout = true;
    }

    pub fn add_children_to(parent: &Rc<RefCell<Widget<'a>>>,
                        children: Vec<Rc<RefCell<Widget<'a>>>>) {
        for child in children.into_iter() {
            Widget::add_child_to(parent, child);
        }
    }

    pub fn get_child_with_name(widget: &Rc<RefCell<Widget<'a>>>,
                               name: &str) -> Option<Rc<RefCell<Widget<'a>>>> {
        for child in widget.borrow().children.iter() {
            if child.borrow().kind.get_name() == name {
                return Some(Rc::clone(child));
            }
        }
        None
    }

    pub fn update(root: &Rc<RefCell<Widget<'a>>>) -> Result<(), Error> {
        Widget::check_children(&root)?;

        root.borrow_mut().layout_widget();

        Ok(())
    }

    pub fn check_children(parent: &Rc<RefCell<Widget<'a>>>) -> Result<(), Error> {
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

        // set up theme
        if parent.borrow().theme.is_none() {
            let parent_parent = Widget::get_parent(parent);

            let parent_parent_theme = match parent_parent.borrow().theme {
                None => return Err(Error::new(ErrorKind::InvalidData,
                    format!("No theme exists for {}", parent_parent.borrow().kind.get_name()))),
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

    pub fn dispatch_event(widget: &Rc<RefCell<Widget<'a>>>,
                          state: &mut GameState, event: Event) -> bool {
        trace!("Dispatching event {:?} in {:?}", event,
               widget.borrow().theme_id);

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
        let mut event_eaten = false;

        let len = widget.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(widget.borrow().children.get(i).unwrap());

            if child.borrow().state.in_bounds(event.mouse) {
                if !child.borrow().state.mouse_is_inside {
                    trace!("Dispatch mouse entered to '{}'", child.borrow().theme_id);
                    Widget::dispatch_event(&child, state, Event::entered_from(&event));
                }

                if !event_eaten && Widget::dispatch_event(&child, state, event) {
                    event_eaten = true;
                }
            } else if child.borrow().state.mouse_is_inside {
                trace!("Dispatch mouse exited to '{}'", child.borrow().theme_id);
                Widget::dispatch_event(&child, state, Event::exited_from(&event));
            }
        }

        if event_eaten { return true; }

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
