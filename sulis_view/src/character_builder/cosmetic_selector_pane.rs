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

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{Widget, WidgetKind};
use sulis_widgets::{Label};

use CharacterBuilder;
use character_builder::BuilderPane;

pub const NAME: &str = "cosmetic_selector_pane";

pub struct CosmeticSelectorPane {
}

impl CosmeticSelectorPane {
    pub fn new() -> Rc<RefCell<CosmeticSelectorPane>> {
        Rc::new(RefCell::new(CosmeticSelectorPane {
        }))
    }
}

impl BuilderPane for CosmeticSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder) {
        builder.prev.borrow_mut().state.set_enabled(true);
        builder.next.borrow_mut().state.set_enabled(false);
        builder.next.borrow_mut().state.set_visible(false);
        builder.finish.borrow_mut().state.set_visible(true);
    }

    fn next(&mut self, _builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
        //builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.next.borrow_mut().state.set_visible(true);
        builder.finish.borrow_mut().state.set_visible(false);
        builder.prev(&widget);
    }
}

impl WidgetKind for CosmeticSelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        vec![title]
    }
}
