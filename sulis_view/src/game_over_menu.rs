//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_state::GameState;
use sulis_widgets::{Button, Label};

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
