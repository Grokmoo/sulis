use std::rc::Rc;
use std::cell::RefCell;

use rules::Attribute;
use state::{EntityState};
use grt::ui::{Button, Callback, Label, TextArea, Widget, WidgetKind};

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
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.id);

        let close = Widget::with_theme(Button::with_callback(Callback::remove_parent()), "close");

        let details = Widget::with_theme(TextArea::empty(), "details");
        {
            let pc = self.character.borrow();
            let state = &mut details.borrow_mut().state;
            state.add_text_arg("name", &pc.actor.actor.name);
            state.add_text_arg("race", &pc.actor.actor.race.name);
            state.add_text_arg("sex", &pc.actor.actor.sex.to_string());

            for attribute in Attribute::iter() {
                state.add_text_arg(attribute.short_name(), &pc.actor.attributes.get(attribute).to_string())
            }
        }
        vec![title, close, details]
    }
}
