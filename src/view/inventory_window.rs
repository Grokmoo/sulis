use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState, ChangeListener, GameState};
use grt::ui::{AnimationState, Callback, Button, Label, ListBox, Widget, WidgetKind};
use grt::ui::{list_box, animation_state};

pub const NAME: &str = "inventory_window";

pub struct InventoryWindow {
    entity: Rc<RefCell<EntityState>>,
}

impl InventoryWindow {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<InventoryWindow> {
        Rc::new(InventoryWindow {
            entity: Rc::clone(entity)
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

    fn on_remove(&self) {
        self.entity.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed inventory window.");
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let ref actor = self.entity.borrow().actor;

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        for (index, item) in actor.inventory().items.iter().enumerate() {
            let cb = match item.item.equippable {
                Some(equippable) => {
                    let slot = equippable.slot;
                    Some(Callback::with(Box::new(move || {
                        let pc = GameState::pc();
                        let mut pc = pc.borrow_mut();

                        if pc.actor.inventory().is_equipped(index) {
                            pc.actor.unequip(slot);
                        } else {
                            pc.actor.equip(index);
                        }
                    })))
                },
                None => None,
            };

            let entry = if actor.inventory().is_equipped(index) {
                list_box::Entry::with_state(item.item.name.to_string(), cb,
                    AnimationState::with(animation_state::Kind::Active))
            } else {
                list_box::Entry::new(item.item.name.to_string(), cb)
            };

            entries.push(entry);
        }

        let list = Widget::with_theme(ListBox::new(entries), "inventory");

        vec![title, close, list]
    }
}
