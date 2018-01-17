use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Label, Widget, WidgetKind};
use state::{ChangeListener, GameState};

pub const NAME: &str = "initiative_ticker";

pub struct InitiativeTicker { }

impl InitiativeTicker {
    pub fn new() -> Rc<InitiativeTicker> {
        Rc::new(InitiativeTicker {

        })
    }
}

impl WidgetKind for InitiativeTicker {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_remove(&self) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        area_state.turn_timer.listeners.remove(NAME);
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        let timer = &mut area_state.turn_timer;
        timer.listeners.add(ChangeListener::invalidate(NAME, widget));

        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();
        let mut first = true;
        for entity in timer.iter() {
            let theme = match first {
                true => "current_entry",
                false => "entry",
            };
            let widget = Widget::with_theme(Label::new(&entity.borrow().actor.actor.name),
                                            theme);
            widgets.push(widget);
            first = false;
        }

        widgets
    }
}
