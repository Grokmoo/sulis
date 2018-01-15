use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState, GameState, Inventory};
use grt::ui::{AnimationState, Callback, Button, Label, ListBox, Widget, WidgetKind};
use grt::ui::{list_box, animation_state};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    inventory: Rc<RefCell<Inventory>>,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<InventoryWindow> {
        Rc::new(InventoryWindow {
           inventory: Rc::clone(&entity.borrow().actor.inventory)
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

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, item) in self.inventory.borrow().items.iter().enumerate() {
            let cb: Callback = Callback::new(Rc::new(move |widget| {
                let pc = GameState::pc();
                let pc = pc.borrow_mut();
                if pc.actor.inventory.borrow_mut().equip(index) {
                    let window = Widget::go_up_tree(widget, 2);
                    window.borrow_mut().invalidate_children();
                }
            }));

            let entry = if self.inventory.borrow().is_equipped(index) {
                list_box::Entry::with_state(item.item.name.to_string(), Some(cb),
                    AnimationState::with(animation_state::Kind::Active))
            } else {
                list_box::Entry::new(item.item.name.to_string(), Some(cb))
            };

            entries.push(entry);
        }

        let list = Widget::with_theme(ListBox::new(entries), "inventory");

        vec![title, close, list]
    }
}
