//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::cell::{Ref, RefCell};
use std::io::Error;
use std::mem;
use std::rc::Rc;

use crate::config::Config;
use crate::io::{event, Event, GraphicsRenderer};
use crate::resource::ResourceSet;
use crate::ui::{theme, Cursor, EmptyWidget, Theme, WidgetKind, WidgetState};
use crate::util::{Point, Size};
use crate::widgets::Label;

pub struct Widget {
    pub state: WidgetState,
    pub kind: Rc<RefCell<WidgetKind>>,
    pub children: Vec<Rc<RefCell<Widget>>>,
    pub theme: Rc<Theme>,
    theme_id: String,
    pub theme_subname: String,

    modal_child: Option<Rc<RefCell<Widget>>>,
    pub(crate) keyboard_focus_child: Option<Rc<RefCell<Widget>>>,
    parent: Option<Rc<RefCell<Widget>>>,

    marked_for_removal: bool,
    marked_for_layout: bool,
    marked_for_readd: bool,
}

impl Widget {
    pub fn theme_id(&self) -> &str {
        &self.theme_id
    }

    pub fn has_modal(&self) -> bool {
        self.modal_child.is_some()
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, pixel_size: Point, millis: u32) {
        if !self.state.visible {
            return;
        }

        if let Some(ref image) = self.state.background {
            let (x, y) = self.state.position().as_tuple();
            let (w, h) = self.state.size().as_tuple();
            image.draw(
                renderer,
                &self.state.animation_state,
                x as f32,
                y as f32,
                w as f32,
                h as f32,
                millis,
            );
        }

        self.kind
            .borrow_mut()
            .draw(renderer, pixel_size, &self, millis);

        for child in self.children.iter() {
            let child = child.borrow();

            child.draw(renderer, pixel_size, millis);

            if let Some(ref image) = child.state.foreground {
                let x = child.state.inner_left() as f32;
                let y = child.state.inner_top() as f32;
                let w = child.state.inner_width() as f32;
                let h = child.state.inner_height() as f32;
                image.draw(renderer, &child.state.animation_state, x, y, w, h, millis);
            }
        }

        self.kind.borrow_mut().end_draw(renderer);
    }

    pub fn set_theme_name(&mut self, name: &str) {
        self.theme_subname = name.to_string();
    }

    pub fn mark_for_removal(&mut self) {
        trace!("Marked widget for removal '{}'", self.theme_id);
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
        trace!("Invalidated widget '{}'", self.theme_id);
        self.marked_for_readd = true;
        for child in self.children.iter_mut() {
            child.borrow_mut().invalidate_children();
        }
        self.marked_for_layout = true;
    }

    pub fn do_base_layout(&mut self) {
        self.do_self_layout();
        self.do_children_layout();
    }

    pub fn do_self_layout(&mut self) {
        let theme = &self.theme;
        theme.apply_background(&mut self.state);
        theme.apply_foreground(&mut self.state);

        if let Some(font) = ResourceSet::font(&theme.text_params.font) {
            self.state.font = Some(font);
        } else if theme.text.is_some() {
            warn!(
                "Font '{}' not found for widget '{}' which has text.",
                theme.text_params.font, self.theme_id
            );
        }

        self.state.set_border(theme.border.clone());
        self.state.text_params = theme.text_params.clone();

        theme.apply_text(&mut self.state);
    }

    pub fn do_children_layout(&self) {
        self.theme.layout.layout(self);
    }

    fn layout_widget(&mut self) {
        if self.marked_for_layout {
            trace!(
                "Performing layout on widget '{}' with size {:?} at {:?}",
                self.theme_id,
                self.state.size(),
                self.state.position()
            );
            let kind = Rc::clone(&self.kind);
            kind.borrow_mut().layout(self);
            self.marked_for_layout = false;

            self.children
                .iter_mut()
                .for_each(|child| child.borrow_mut().marked_for_layout = true);
        }

        let len = self.children.len();
        for i in 0..len {
            let child = Rc::clone(self.children.get(i).unwrap());
            child.borrow_mut().layout_widget();
        }
    }
}

impl Widget {
    fn new(kind: Rc<RefCell<WidgetKind>>, theme: &str) -> Rc<RefCell<Widget>> {
        let widget = Widget {
            state: WidgetState::new(),
            kind: Rc::clone(&kind),
            children: Vec::new(),
            modal_child: None,
            keyboard_focus_child: None,
            parent: None,
            marked_for_layout: true,
            theme: ResourceSet::default_theme(),
            theme_id: String::new(),
            theme_subname: theme.to_string(),
            marked_for_removal: false,
            marked_for_readd: false,
        };

        let widget = Rc::new(RefCell::new(widget));
        let children = kind.borrow_mut().on_add(&widget);
        Widget::add_children_to(&widget, children);

        widget
    }

    pub fn with_defaults(widget: Rc<RefCell<WidgetKind>>) -> Rc<RefCell<Widget>> {
        let name = widget.borrow().get_name().to_string();
        Widget::new(widget, &name)
    }

    pub fn with_theme(widget: Rc<RefCell<WidgetKind>>, theme: &str) -> Rc<RefCell<Widget>> {
        Widget::new(widget, theme)
    }

    pub fn empty(theme: &str) -> Rc<RefCell<Widget>> {
        Widget::new(EmptyWidget::new(), theme)
    }

    pub fn parent_mut<'a, T: WidgetKind + 'static>(
        widget: &'a Rc<RefCell<Widget>>,
    ) -> (Rc<RefCell<Widget>>, &'a mut T) {
        let mut current = Rc::clone(widget);
        loop {
            let kind = Rc::clone(&current.borrow().kind);
            if let Ok(mut kind) = kind.try_borrow_mut() {
                if let Some(kind) = kind.as_any_mut().downcast_mut::<T>() {
                    let kind = unsafe { mem::transmute::<_, &'a mut T>(kind) };
                    return (current, kind);
                }
            }

            let parent = Rc::clone(current.borrow().parent.as_ref().unwrap());
            current = parent;
        }
    }

    pub fn parent<'a, T: WidgetKind + 'static>(
        widget: &'a Rc<RefCell<Widget>>,
    ) -> (Rc<RefCell<Widget>>, &'a T) {
        let mut current = Rc::clone(widget);
        loop {
            let kind = Rc::clone(&current.borrow().kind);
            if let Ok(kind) = kind.try_borrow() {
                if let Some(kind) = kind.as_any().downcast_ref::<T>() {
                    let kind = unsafe { mem::transmute::<_, &'a T>(kind) };
                    return (current, kind);
                }
            }

            let parent = Rc::clone(current.borrow().parent.as_ref().unwrap());
            current = parent;
        }
    }

    pub fn get_root(widget: &Rc<RefCell<Widget>>) -> Rc<RefCell<Widget>> {
        match &widget.borrow().parent {
            None => return Rc::clone(widget),
            Some(parent) => Widget::get_root(parent),
        }
    }

    pub fn kind<'a, T: WidgetKind + 'static>(widget: &'a Rc<RefCell<Widget>>) -> &'a T {
        let kind = Rc::clone(&widget.borrow().kind);
        let kind = kind.borrow();
        let result = match kind.as_any().downcast_ref::<T>() {
            None => panic!("Failed to downcast Kind"),
            Some(result) => result,
        };
        unsafe { mem::transmute::<&T, &'a T>(result) }
    }

    pub fn kind_mut<'a, T: WidgetKind + 'static>(widget: &'a Rc<RefCell<Widget>>) -> &'a mut T {
        let kind = Rc::clone(&widget.borrow().kind);
        let mut kind = kind.borrow_mut();
        let result = match kind.as_any_mut().downcast_mut::<T>() {
            None => panic!("Failed to downcast_mut Kind"),
            Some(result) => result,
        };
        unsafe { mem::transmute::<&mut T, &'a mut T>(result) }
    }
    pub fn downcast<'a, T: WidgetKind + 'static>(kind: &'a WidgetKind) -> &'a T {
        match kind.as_any().downcast_ref::<T>() {
            None => panic!("Failed to downcast kind"),
            Some(result) => result,
        }
    }

    pub fn downcast_mut<'a, T: WidgetKind + 'static>(kind: &'a mut WidgetKind) -> &'a mut T {
        match kind.as_any_mut().downcast_mut::<T>() {
            None => panic!("Failed to downcast kind"),
            Some(result) => result,
        }
    }

    pub fn direct_parent(widget: &Rc<RefCell<Widget>>) -> Rc<RefCell<Widget>> {
        Rc::clone(widget.borrow().parent.as_ref().unwrap())
    }

    fn add_child_to_internal(parent: &Rc<RefCell<Widget>>, child: &Rc<RefCell<Widget>>) {
        {
            let child_ref = child.borrow();
            trace!(
                "Adding {:?} to {:?}",
                child_ref.kind.borrow().get_name(),
                parent.borrow().theme_id
            );

            if child_ref.state.is_modal {
                trace!("Adding child as modal widget.");
                let root = Widget::get_root(parent);
                root.borrow_mut().modal_child = Some(Rc::clone(&child));
                root.borrow_mut().keyboard_focus_child = None;
            }
        }
        parent.borrow_mut().marked_for_layout = true;
    }

    pub fn add_child_to(parent: &Rc<RefCell<Widget>>, child: Rc<RefCell<Widget>>) {
        Widget::add_child_to_internal(parent, &child);
        parent.borrow_mut().children.push(child);
    }

    pub fn add_child_to_front(parent: &Rc<RefCell<Widget>>, child: Rc<RefCell<Widget>>) {
        Widget::add_child_to_internal(parent, &child);
        parent.borrow_mut().children.insert(0, child);
    }

    pub fn add_children_to(parent: &Rc<RefCell<Widget>>, children: Vec<Rc<RefCell<Widget>>>) {
        for child in children.into_iter() {
            Widget::add_child_to(parent, child);
        }
    }

    /// gets the child of the specified widget with the specified kind name, if it
    /// exists.  note that this uses try_borrow and will not check a widget kind
    /// that is already borrowed (typically by the caller).
    /// returns true if the child exists, false otherwise
    pub fn has_child_with_name(widget: &Rc<RefCell<Widget>>, name: &str) -> bool {
        for child in widget.borrow().children.iter() {
            let child_ref = match child.try_borrow() {
                Err(_) => continue,
                Ok(child) => child,
            };

            let kind = match child_ref.kind.try_borrow() {
                Err(_) => continue,
                Ok(kind) => kind,
            };

            if kind.get_name() == name {
                return true;
            }
        }
        false
    }

    /// gets the child of the specified widget with the specified kind name, if it
    /// exists.  note that this uses try_borrow and will not check a widget kind
    /// that is already borrowed (typically by the caller)
    pub fn get_child_with_name(
        widget: &Rc<RefCell<Widget>>,
        name: &str,
    ) -> Option<Rc<RefCell<Widget>>> {
        for child in widget.borrow().children.iter() {
            let child_ref = match child.try_borrow() {
                Err(_) => continue,
                Ok(child) => child,
            };

            let kind = match child_ref.kind.try_borrow() {
                Err(_) => continue,
                Ok(kind) => kind,
            };

            if kind.get_name() == name {
                return Some(Rc::clone(child));
            }
        }
        None
    }

    /// Attempts to grab keyboard focus.  this will fail if
    /// the widget has not been added to the tree yet
    pub fn grab_keyboard_focus(widget: &Rc<RefCell<Widget>>) -> bool {
        let root = Widget::get_root(widget);
        if Rc::ptr_eq(&root, widget) {
            return false;
        }
        Widget::remove_old_keyboard_focus(&root);
        root.borrow_mut().keyboard_focus_child = Some(Rc::clone(widget));
        widget.borrow_mut().state.has_keyboard_focus = true;
        trace!("Keyboard focus to {}", widget.borrow().theme_id);
        true
    }

    pub fn clear_keyboard_focus(widget: &Rc<RefCell<Widget>>) {
        let root = Widget::get_root(widget);
        Widget::remove_old_keyboard_focus(&root);
        trace!("Cleared keyboard focus");
    }

    fn remove_old_keyboard_focus(root: &Rc<RefCell<Widget>>) {
        let mut root = root.borrow_mut();

        if root.keyboard_focus_child.is_none() {
            return;
        }

        {
            let child = &root.keyboard_focus_child.as_ref().unwrap();
            child.borrow_mut().state.has_keyboard_focus = false;
        }
        root.keyboard_focus_child = None;
    }

    pub fn fire_callback(widget: &Rc<RefCell<Widget>>, kind: &mut WidgetKind) {
        let cb = match widget.borrow().state.callback {
            None => return,
            Some(ref cb) => cb.clone(),
        };

        (cb).call(widget, kind);
    }

    pub fn remove_mouse_over(root: &Rc<RefCell<Widget>>) {
        for child in root.borrow().children.iter() {
            if !child.borrow().state.is_mouse_over {
                continue;
            }
            child.borrow_mut().mark_for_removal();
        }
    }

    pub fn set_mouse_over_widget(
        widget: &Rc<RefCell<Widget>>,
        mouse_over: Rc<RefCell<Widget>>,
        x: i32,
        y: i32,
    ) {
        let root = Widget::get_root(widget);
        Widget::remove_mouse_over(&root);

        trace!("Add mouse over from '{}'", widget.borrow().theme_id);
        mouse_over.borrow_mut().state.is_mouse_over = true;
        mouse_over.borrow_mut().state.set_position(x, y);
        Widget::add_child_to(&root, mouse_over);
    }

    pub fn set_mouse_over(
        widget: &Rc<RefCell<Widget>>,
        mouse_over: Rc<RefCell<WidgetKind>>,
        x: i32,
        y: i32,
    ) {
        let mouse_over = Widget::with_defaults(mouse_over);
        Widget::set_mouse_over_widget(widget, mouse_over, x, y);
    }

    pub fn update(root: &Rc<RefCell<Widget>>, millis: u32) -> Result<(), Error> {
        Widget::update_kind_recursive(&root, millis);

        let mut find_new_modal = false;
        if let Some(ref child) = root.borrow().modal_child {
            if child.borrow().marked_for_removal {
                find_new_modal = true;
            }
        }

        Widget::check_children_removal(&root);

        if find_new_modal {
            let modal = Widget::find_new_modal_child(root);
            root.borrow_mut().modal_child = modal;
        }

        Widget::check_readd(&root);
        Widget::check_children(&root)?;

        root.borrow_mut().layout_widget();

        Ok(())
    }

    fn update_kind_recursive(widget: &Rc<RefCell<Widget>>, millis: u32) {
        let kind = Rc::clone(&widget.borrow().kind);
        kind.borrow_mut().update(&widget, millis);

        let len = widget.borrow().children.len();
        for i in 0..len {
            let child = Rc::clone(&widget.borrow().children[i]);
            Widget::update_kind_recursive(&child, millis);
        }
    }

    pub fn check_readd(parent: &Rc<RefCell<Widget>>) {
        let readd = parent.borrow().marked_for_readd;
        if readd {
            parent.borrow_mut().modal_child = None;
            for child in parent.borrow_mut().children.iter() {
                let child_ref = child.borrow_mut();
                child_ref.kind.borrow_mut().on_remove();
            }
            parent.borrow_mut().children.clear();
            let kind = Rc::clone(&parent.borrow().kind);
            kind.borrow_mut().on_remove();
            let children = kind.borrow_mut().on_add(&parent);
            Widget::add_children_to(parent, children);
            parent.borrow_mut().marked_for_readd = false;
            parent.borrow_mut().marked_for_layout = true;
            if parent.borrow().parent.is_some() {
                // don't try to force layout on the root widget
                parent.borrow_mut().theme_id = String::new();
            }
        } else {
            let len = parent.borrow().children.len();
            for i in 0..len {
                let child = Rc::clone(parent.borrow().children.get(i).unwrap());
                Widget::check_readd(&child);
            }
        }
    }

    fn recursive_on_remove(widget_ref: &Ref<Widget>) {
        let len = widget_ref.children.len();
        for i in 0..len {
            let child = Rc::clone(&widget_ref.children[i]);
            Widget::recursive_on_remove(&child.borrow());
        }

        widget_ref.kind.borrow_mut().on_remove();
    }

    fn find_new_modal_child(parent: &Rc<RefCell<Widget>>) -> Option<Rc<RefCell<Widget>>> {
        let len = parent.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(&parent.borrow().children[i]);
            if child.borrow().state.is_modal {
                return Some(child);
            }

            if let Some(child) = Widget::find_new_modal_child(&child) {
                return Some(child);
            }
        }

        None
    }

    pub fn check_children_removal(parent: &Rc<RefCell<Widget>>) {
        parent.borrow_mut().children.retain(|widget| {
            let widget_ref = widget.borrow();
            if widget_ref.marked_for_removal {
                Widget::recursive_on_remove(&widget_ref);
                false
            } else {
                true
            }
        });

        let parent = parent.borrow();
        let len = parent.children.len();
        for i in 0..len {
            let child = Rc::clone(&parent.children[i]);
            Widget::check_children_removal(&child);
        }
    }

    pub(in crate::ui) fn setup_root(root: &Rc<RefCell<Widget>>) {
        let (ui_x, ui_y) = Config::ui_size();
        let mut root = root.borrow_mut();
        root.state.set_size(Size::new(ui_x, ui_y));
        root.theme_id = root.theme_subname.clone();
        root.theme = ResourceSet::theme(&root.theme_id);
    }

    pub fn check_children(parent: &Rc<RefCell<Widget>>) -> Result<(), Error> {
        // set up theme
        if parent.borrow().theme_id.is_empty() {
            let parent_parent = Widget::direct_parent(parent);

            let mut parent = parent.borrow_mut();
            parent.theme_id = ResourceSet::compute_theme_id(
                &parent_parent.borrow().theme_id,
                &parent.theme_subname,
            );

            let theme = ResourceSet::theme(&parent.theme_id);
            match theme.kind {
                theme::Kind::Ref => (),
                _ => warn!(
                    "Manually added widget for theme {} with kind {:?}",
                    theme.id, theme.kind
                ),
            }

            // add theme specified children
            for id in theme.children.iter() {
                let subname = match id.split(".").last() {
                    None => {
                        warn!("Invalid theme child name {}", id);
                        continue;
                    }
                    Some(name) => name,
                };
                let child_theme = ResourceSet::theme(id);
                let child;
                match child_theme.kind {
                    theme::Kind::Label => child = Widget::with_theme(Label::empty(), subname),
                    theme::Kind::Container => child = Widget::empty(subname),
                    theme::Kind::Ref => continue,
                }

                {
                    let mut child = child.borrow_mut();
                    child.theme_id = id.clone();
                    child.theme = child_theme;
                    child.marked_for_layout = true;
                }
                parent.children.push(child);
            }

            parent.theme = theme;
        }

        // set up parent references
        let len = parent.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(parent.borrow().children.get(i).unwrap());
            if child.borrow().parent.is_none() {
                child.borrow_mut().parent = Some(Rc::clone(parent));
            }

            Widget::check_children(&child)?;
        }

        Ok(())
    }

    pub fn dispatch_event(widget: &Rc<RefCell<Widget>>, event: Event) -> bool {
        if widget.borrow().state.is_mouse_over {
            return false;
        }

        match event.kind {
            event::Kind::MouseMove { .. } => (),
            _ => trace!(
                "Dispatching event {:?} in {:?}",
                event,
                widget.borrow().theme_id
            ),
        }

        let ref widget_kind = Rc::clone(&widget.borrow().kind);

        let has_keyboard_child = widget.borrow().keyboard_focus_child.is_some();
        if has_keyboard_child {
            let child = Rc::clone(widget.borrow().keyboard_focus_child.as_ref().unwrap());
            let child_kind = Rc::clone(&child.borrow().kind);

            trace!("Dispatch to keyboard focus child: {}", child.borrow().theme_id);
            match event.kind {
                event::Kind::CharTyped(c) => {
                    return child_kind.borrow_mut().on_char_typed(&child, c);
                }
                event::Kind::KeyPress(key) => {
                    // send key press events only to the keyboard focus child when one exists
                    return child_kind.borrow_mut().on_key_press(&child, key);
                }
                _ => (),
            }
        } else {
            match event.kind {
                event::Kind::CharTyped(_) => return false,
                _ => (),
            }
        }

        let has_modal = widget.borrow().modal_child.is_some();
        if has_modal {
            let child = Rc::clone(widget.borrow().modal_child.as_ref().unwrap());
            trace!("Dispatching to modal child: {}", child.borrow().theme_id);
            return Widget::dispatch_event(&child, event);
        }

        // iterate in this way using indices so we don't maintain any
        // borrows except for the active child widget - this will allow
        // the child to mutate any other widget in the tree
        let mut event_eaten = false;

        let len = widget.borrow().children.len();
        for i in (0..len).rev() {
            let child = Rc::clone(&widget.borrow().children[i]);

            if !child.borrow().state.is_enabled() || !child.borrow().state.visible {
                continue;
            }

            if child
                .borrow()
                .state
                .in_bounds(Cursor::get_x(), Cursor::get_y())
            {
                if !child.borrow().state.mouse_is_inside {
                    trace!("Dispatch mouse entered to '{}'", child.borrow().theme_id);
                    Widget::dispatch_event(&child, Event::entered_from(&event));
                }

                if !event_eaten && Widget::dispatch_event(&child, event) {
                    trace!("Event dispatch eaten by child.");
                    event_eaten = true;
                }
            } else if child.borrow().state.mouse_is_inside {
                trace!("Dispatch mouse exited to '{}'", child.borrow().theme_id);
                Widget::dispatch_event(&child, Event::exited_from(&event));
            }
        }

        use crate::io::event::Kind::*;
        // always pass mouse entered and exited to the widget kind
        let enter_exit_retval = match event.kind {
            MouseEnter => widget_kind.borrow_mut().on_mouse_enter(widget),
            MouseExit => widget_kind.borrow_mut().on_mouse_exit(widget),
            _ => false,
        };

        // don't pass events other than mouse enter, exit to the widget kind
        // if a child ate the event
        if event_eaten {
            return true;
        }

        trace!("Dispatch to direct widget kind.");
        match event.kind {
            MousePress(kind) => widget_kind.borrow_mut().on_mouse_press(widget, kind),
            MouseRelease(kind) => widget_kind.borrow_mut().on_mouse_release(widget, kind),
            MouseMove { delta_x, delta_y } => widget_kind
                .borrow_mut()
                .on_mouse_move(widget, delta_x, delta_y),
            MouseDrag {
                button: kind,
                delta_x,
                delta_y,
            } => widget_kind
                .borrow_mut()
                .on_mouse_drag(widget, kind, delta_x, delta_y),
            KeyPress(action) => widget_kind.borrow_mut().on_key_press(widget, action),
            RawKey(key) => widget_kind.borrow_mut().on_raw_key(widget, key),
            MouseEnter => enter_exit_retval,
            MouseExit => enter_exit_retval,
            _ => false,
        }
    }
}
