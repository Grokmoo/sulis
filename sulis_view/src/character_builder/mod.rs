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

mod attribute_selector_pane;
use self::attribute_selector_pane::AttributeSelectorPane;

mod class_selector_pane;
use self::class_selector_pane::ClassSelectorPane;

mod cosmetic_selector_pane;
use self::cosmetic_selector_pane::CosmeticSelectorPane;

mod race_selector_pane;
use self::race_selector_pane::RaceSelectorPane;

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};

pub const NAME: &str = "character_builder";

trait BuilderPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder);

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>);

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>);
}

pub struct CharacterBuilder {
    pub (in character_builder) next: Rc<RefCell<Widget>>,
    pub (in character_builder) prev: Rc<RefCell<Widget>>,
    pub (in character_builder) finish: Rc<RefCell<Widget>>,
    builder_panes: Vec<Rc<RefCell<BuilderPane>>>,
    builder_pane_index: usize,
    // we rely on the builder panes in the above vec having the same
    // index in the children vec of this widget
}

impl CharacterBuilder {
    pub fn new() -> Rc<RefCell<CharacterBuilder>> {
        let next = Widget::with_theme(Button::empty(), "next");
        next.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&parent);
            let cur_pane = Rc::clone(&builder.builder_panes[builder.builder_pane_index]);
            cur_pane.borrow_mut().next(builder, Rc::clone(&parent));
        })));

        let prev = Widget::with_theme(Button::empty(), "previous");
        prev.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(widget);
            let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&parent);
            let cur_pane = Rc::clone(&builder.builder_panes[builder.builder_pane_index]);
            cur_pane.borrow_mut().prev(builder, Rc::clone(&parent));
        })));

        let finish = Widget::with_theme(Button::empty(), "finish");
        finish.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {

        })));

        Rc::new(RefCell::new(CharacterBuilder {
            next,
            prev,
            finish,
            builder_panes: Vec::new(),
            builder_pane_index: 0,
        }))
    }

    pub fn next(&mut self, widget: &Rc<RefCell<Widget>>) {
        self.change_index(widget, 1);
    }

    pub fn prev(&mut self, widget: &Rc<RefCell<Widget>>) {
        self.change_index(widget, -1);
    }

    fn change_index(&mut self, widget: &Rc<RefCell<Widget>>, delta: i32) {
        self.set_cur_child_visible(widget, false);
        self.builder_pane_index = (self.builder_pane_index as i32 + delta) as usize;
        let cur_pane = Rc::clone(&self.builder_panes[self.builder_pane_index]);
        cur_pane.borrow_mut().on_selected(self);
        self.set_cur_child_visible(widget, true);
    }

    fn set_cur_child_visible(&self, widget: &Rc<RefCell<Widget>>, vis: bool) {
        let cur_child = Rc::clone(&widget.borrow().children[self.builder_pane_index]);
        cur_child.borrow_mut().state.set_visible(vis);
    }
}

impl WidgetKind for CharacterBuilder {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let race_selector_pane = RaceSelectorPane::new();
        let class_selector_pane = ClassSelectorPane::new();
        let attribute_selector_pane = AttributeSelectorPane::new();
        let cosmetic_selector_pane = CosmeticSelectorPane::new();
        let race_sel_widget = Widget::with_defaults(race_selector_pane.clone());
        let class_sel_widget = Widget::with_defaults(class_selector_pane.clone());
        let attr_sel_widget = Widget::with_defaults(attribute_selector_pane.clone());
        let cosmetic_sel_widget = Widget::with_defaults(cosmetic_selector_pane.clone());
        class_sel_widget.borrow_mut().state.set_visible(false);
        attr_sel_widget.borrow_mut().state.set_visible(false);
        cosmetic_sel_widget.borrow_mut().state.set_visible(false);
        self.finish.borrow_mut().state.set_visible(false);

        self.builder_panes.clear();
        self.builder_pane_index = 0;
        self.builder_panes.push(race_selector_pane.clone());
        self.builder_panes.push(class_selector_pane.clone());
        self.builder_panes.push(attribute_selector_pane.clone());
        self.builder_panes.push(cosmetic_selector_pane.clone());

        race_selector_pane.borrow_mut().on_selected(self);

        vec![race_sel_widget, class_sel_widget, attr_sel_widget, cosmetic_sel_widget, title, close,
            Rc::clone(&self.next), Rc::clone(&self.prev), Rc::clone(&self.finish)]
    }
}
