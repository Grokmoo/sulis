use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState};
use grt::ui::{Button, Callback, Label, Widget, WidgetKind};

pub const NAME: &str = "character_window";

pub struct CharacterWindow {
    character: Rc<RefCell<EntityState>>,
}

impl CharacterWindow {
    pub fn new(character: &Rc<RefCell<EntityState>>) -> Rc<CharacterWindow> {
        Rc::new(CharacterWindow {
            character: Rc::clone(character)
        })
    }
}

impl WidgetKind for CharacterWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg(&self.character.borrow().actor.actor.id);

        let close = Widget::with_theme(Button::with_callback(Callback::remove_parent()), "close");

        vec![title, close]
    }
}
