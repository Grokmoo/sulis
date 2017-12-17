use std::rc::Rc;
use std::cell::RefCell;

use ui::{Button, Label, Widget, WidgetKind};

pub struct InventoryWindow {

}

impl InventoryWindow {
    pub fn new() -> Rc<InventoryWindow> {
        Rc::new(InventoryWindow {

        })
    }
}

impl<'a> WidgetKind<'a> for InventoryWindow {
    fn get_name(&self) -> &str {
        "inventory_window"
    }

    fn layout(&self, widget: &mut Widget<'a>) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(
            Button::new(Box::new(|widget, _state| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            "close");

        vec![title, close]
    }
}
