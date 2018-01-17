use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Label, Widget, WidgetKind};
use grt::util::Size;
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

    fn layout(&self, widget: &mut Widget) {
        widget.do_self_layout();

        let width = widget.state.inner_size.width;
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            let theme = match child.borrow().theme {
                None => continue,
                Some(ref t) => Rc::clone(t),
            };
            let height = theme.preferred_size.height;
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
        }
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
