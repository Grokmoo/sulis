use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState, Inventory};
use ui::{AnimationState, Callback, Button, Label, ListBox, Widget, WidgetKind};
use ui::list_box;

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    inventory: Inventory,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<InventoryWindow> {
        Rc::new(InventoryWindow {
           inventory: entity.borrow().actor.inventory.clone()
        })
    }
}

impl WidgetKind for InventoryWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(
            Button::with_callback(Rc::new(|_kind, widget, _state| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            })),
            "close");

        let mut entries: Vec<list_box::Entry> = Vec::new();
        for (index, item) in self.inventory.items.iter().enumerate() {
            let cb: Callback<Button> = Rc::new(move |_, widget, state| {
                if state.pc_mut().actor.inventory.equip(index) {
                    widget.borrow_mut().state.append_text(" - equipped");
                    widget.borrow_mut().invalidate_layout();
                    // TODO set active animation state for equipped items
                    // it is being reset by mouseover currently
                }
            });

            let entry = if self.inventory.is_equipped(index) {
                list_box::Entry::with_state(&item.item.name, Some(cb), AnimationState::Active)
            } else {
                list_box::Entry::new(&item.item.name, Some(cb))
            };

            entries.push(entry);
        }

        let list = Widget::with_theme(ListBox::new(entries), "inventory");

        vec![title, close, list]
    }
}
