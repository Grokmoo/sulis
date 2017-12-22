use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState, Inventory};
use ui::{Button, Label, ListBox, Widget, WidgetKind};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    inventory: Inventory,
}

impl<'a> InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState<'a>>>) -> Rc<InventoryWindow> {
        Rc::new(InventoryWindow {
           inventory: entity.borrow().actor.inventory.clone()
        })
    }
}

impl<'a> WidgetKind<'a> for InventoryWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn layout(&self, widget: &mut Widget<'a>) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(
            Button::with_callback(Box::new(|widget, _state| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            "close");

        let mut entries: Vec<String> = Vec::new();
        for item in self.inventory.items.iter() {
            entries.push(item.item.name.clone());
        }

        let list = Widget::with_theme(ListBox::new(entries), "inventory");

        vec![title, close, list]
    }
}
