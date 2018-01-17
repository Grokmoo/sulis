use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Button, Callback, Label, Widget, WidgetKind};
use state::GameState;

const NAME: &str = "game_over_menu";

pub struct GameOverMenu {

}

impl GameOverMenu {
    pub fn new() -> Rc<GameOverMenu> {
        Rc::new(GameOverMenu {

        })
    }
}

impl WidgetKind for GameOverMenu {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let label = Widget::with_theme(Label::empty(), "title");
        let exit = Widget::with_theme(Button::empty(), "exit");
        exit.borrow_mut().state.add_callback(Callback::with(Box::new(|| {
            GameState::set_exit();
        })));

        vec![label, exit]
    }
}
