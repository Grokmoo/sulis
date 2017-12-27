use std::rc::Rc;
use std::cell::RefCell;

use state::{EntityState};
use grt::ui::{Label, Widget, WidgetKind};

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

        vec![title]
    }
}
