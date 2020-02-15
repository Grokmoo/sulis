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
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::ui::{Widget, WidgetKind, RcRfc};
use sulis_core::widgets::Label;

pub const NAME: &str = "loading_screen";

pub struct LoadingScreen {}

impl LoadingScreen {
    pub fn new() -> RcRfc<LoadingScreen> {
        Rc::new(RefCell::new(LoadingScreen {}))
    }
}

impl WidgetKind for LoadingScreen {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let loading_label = Widget::with_theme(Label::empty(), "loading_label");
        let background = Widget::empty("background");
        vec![background, loading_label]
    }
}
