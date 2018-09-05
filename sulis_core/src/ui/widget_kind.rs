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

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use io::{GraphicsRenderer, InputAction, keyboard_event::Key};
use io::event::ClickKind;
use ui::{animation_state, Cursor, Widget};
use util::Point;

pub (crate) struct EmptyWidget { }

impl EmptyWidget {
    pub fn new() -> Rc<RefCell<EmptyWidget>> {
        Rc::new(RefCell::new(EmptyWidget {}))
    }
}

impl WidgetKind for EmptyWidget {
    widget_kind!["content"];

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(&mut self, _widget: &Rc<RefCell<Widget>>, _kind: ClickKind,
                     _delta_x: f32, _delta_y: f32) -> bool {
        false
    }

    fn on_mouse_move(&mut self, _widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        false
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }
}

/// Trait for implementations of different Widgets.  This is held by a 'WidgetState'
/// object which contains the common functionality across all Widgets.
pub trait WidgetKind {
    /// called every frame
    fn update(&mut self, _widget: &Rc<RefCell<Widget>>, _millis: u32) { }

    fn draw(&mut self, _renderer: &mut GraphicsRenderer, _pixel_size: Point,
            _widget: &Widget, _millis: u32) { }

    fn end_draw(&mut self, _renderer: &mut GraphicsRenderer) { }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn get_name(&self) -> &str;

    fn as_any(&self) -> &Any;

    fn as_any_mut(&mut self) -> &mut Any;

    /// Whether this Widget kind wants keyboard focus.  When a widget is receiving keyboard
    /// focus, it gets character events, allowing the user to type into the widget.  By default,
    /// a widget will receive keyboard focus when clicked and lose it when another widget is
    /// clicked.
    fn wants_keyboard_focus(&self) -> bool { false }

    /// This method is called before this WidgetKind is added to its parent widget.
    /// It returns a vector of 'Widget's that will be added as children to the
    /// parent widget.  If you implement this but do not need to add any children,
    /// returning 'Vec::with_capacity(0)' is best.
    /// Widgets are added according to their order in the vector.
    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        Vec::with_capacity(0)
    }

    /// This method is called just prior to the owning widget being removed from
    /// the widget tree.  It is used for any cleanup that needs to be done
    /// when a widget is removed from the tree.  Note that this is called when
    /// the actual removal takes place, not when the widget is first marked
    /// for removal.
    fn on_remove(&mut self) { }

    fn super_on_mouse_press(&self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) {
        widget.borrow_mut().state.animation_state.add(animation_state::Kind::Pressed);

        if self.wants_keyboard_focus() {
            Widget::grab_keyboard_focus(widget);
        } else {
            Widget::clear_keyboard_focus(widget);
        }
    }

    fn super_on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) {
        widget.borrow_mut().state.animation_state.remove(animation_state::Kind::Pressed);

        if !widget.borrow().state.modal_remove_on_click_outside || !widget.borrow().state.is_modal {
            return;
        }

        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }
    }

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        true
    }

    fn on_mouse_drag(&mut self, _widget: &Rc<RefCell<Widget>>, _kind: ClickKind,
                     _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn on_mouse_move(&mut self, _widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        true
    }

    fn on_raw_key(&mut self, _widget: &Rc<RefCell<Widget>>, _key: Key) -> bool {
        false
    }

    fn on_key_press(&mut self, _widget: &Rc<RefCell<Widget>>, _key: InputAction) -> bool {
        false
    }

    fn on_char_typed(&mut self, _widget: &Rc<RefCell<Widget>>, _c: char) -> bool {
        false
    }

    fn super_on_mouse_enter(&self, widget: &Rc<RefCell<Widget>>) {
        widget.borrow_mut().state.set_mouse_inside(true);
        widget.borrow_mut().state.animation_state.add(animation_state::Kind::Hover);
        trace!("Mouse entered '{}', anim state: '{:?}'",
               widget.borrow().theme_id, widget.borrow().state.animation_state);
    }

    fn super_on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>) {
        widget.borrow_mut().state.set_mouse_inside(false);
        widget.borrow_mut().state.animation_state.remove(animation_state::Kind::Hover);
        widget.borrow_mut().state.animation_state.remove(animation_state::Kind::Pressed);
        trace!("Mouse exited '{}', anim state: '{:?}'",
               widget.borrow().theme_id, widget.borrow().state.animation_state);
    }
}
