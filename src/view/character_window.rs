use std::rc::Rc;
use std::cell::RefCell;

use rules::Attribute;
use state::{ChangeListener, EntityState};
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

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&self) {
        self.character.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed character window.");
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.character.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.id);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

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

            let mut index = 0;
            for &(ref class, level) in pc.actor.actor.levels.iter() {
                state.add_text_arg(&format!("class_{}", index), &class.name);
                state.add_text_arg(&format!("level_{}", index), &level.to_string());

                index += 1;
            }

            state.add_text_arg("damage_min", &pc.actor.stats.damage.min.to_string());
            state.add_text_arg("damage_max", &pc.actor.stats.damage.max.to_string());

            state.add_text_arg("cur_hp", &pc.actor.hp().to_string());
            state.add_text_arg("max_hp", &pc.actor.stats.max_hp.to_string());
        }
        vec![title, close, details]
    }
}
