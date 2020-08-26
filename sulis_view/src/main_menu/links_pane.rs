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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label};

pub struct LinksPane {}

impl LinksPane {
    pub fn new() -> Rc<RefCell<LinksPane>> {
        Rc::new(RefCell::new(LinksPane {}))
    }
}

impl WidgetKind for LinksPane {
    widget_kind!("links_pane");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let credits = Widget::with_theme(Button::empty(), "credits");
        credits
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|_, _| {
                open_link("https://github.com/Grokmoo/sulis/blob/master/docs/attribution.csv");
            })));

        let github = Widget::with_theme(Button::empty(), "github");
        github
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|_, _| {
                open_link("https://github.com/Grokmoo/sulis");
            })));

        let changes = Widget::with_theme(Button::empty(), "changes");
        changes
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|_, _| {
                open_link("https://github.com/Grokmoo/sulis/blob/master/Changelog.md");
            })));

        let website = Widget::with_theme(Button::empty(), "website");
        website
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|_, _| {
                open_link("https://www.sulisgame.com");
            })));

        vec![title, credits, github, changes, website]
    }
}

fn open_link(link: &str) {
    let thread = open::that_in_background(link);

    if thread.join().is_err() {
        warn!("Unable to open link using web browser: {}", link);
    }
}
