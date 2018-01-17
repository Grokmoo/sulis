use std::rc::Rc;
use std::cell::RefCell;

use io::{GraphicsRenderer, InputAction, TextRenderer};
use io::event::ClickKind;
use ui::{animation_state, Widget};
use util::Point;

pub struct EmptyWidget { }

impl EmptyWidget {
    pub fn new() -> Rc<EmptyWidget> {
        Rc::new(EmptyWidget {})
    }
}

impl WidgetKind for EmptyWidget {
    fn get_name(&self) -> &str {
        "content"
    }
}

/// Trait for implementations of different Widgets.  This is held by a 'WidgetState'
/// object which contains the common functionality across all Widgets.
pub trait WidgetKind {
    /// called every frame
    fn update(&self, _widget: &Rc<RefCell<Widget>>) { }

    fn draw_text_mode(&self, _renderer: &mut TextRenderer,
                      _widget: &Widget, _millis: u32) { }

    fn draw_graphics_mode(&self, _renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          _widget: &Widget, _millis: u32) { }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn get_name(&self) -> &str;

    /// This method is called after this WidgetKind is added to its parent widget.
    /// It returns a vector of 'Widget's that will be added as children to the
    /// parent widget.  If you implement this but do not need to add any children,
    /// returning 'Vec::with_capacity(0)' is best.
    /// Widgets are added according to their order in the vector.
    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        Vec::with_capacity(0)
    }

    /// This method is called just prior to the owning widget being removed from
    /// the widget tree.  It is used for any cleanup that needs to be done
    /// when a widget is removed from the tree.  Note that this is called when
    /// the actual removal takes place, not when the widget is first marked
    /// for removal.
    fn on_remove(&self) { }

    fn super_on_mouse_press(&self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) {
        widget.borrow_mut().state.animation_state.add(animation_state::Kind::Pressed);
    }

    fn super_on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) {
        widget.borrow_mut().state.animation_state.remove(animation_state::Kind::Pressed);
    }

    fn on_mouse_press(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        true
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        true
    }

    fn on_mouse_move(&self, _widget: &Rc<RefCell<Widget>>) -> bool {
        true
    }

    fn on_mouse_enter(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        true
    }

    fn on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        true
    }

    fn on_mouse_scroll(&self, _widget: &Rc<RefCell<Widget>>, _scroll: i32) -> bool {
        true
    }

    fn on_key_press(&self, _widget: &Rc<RefCell<Widget>>, _key: InputAction) -> bool {
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
