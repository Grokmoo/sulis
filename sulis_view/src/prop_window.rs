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

use sulis_state::{ChangeListener, GameState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use RootView;
use inventory_window::ItemButton;

pub const NAME: &str = "prop_window";

pub struct PropWindow {
    prop_index: usize,
}

impl PropWindow {
    pub fn new(prop_index: usize) -> Rc<RefCell<PropWindow>> {
        Rc::new(RefCell::new(PropWindow {
            prop_index,
        }))
    }
}

impl WidgetKind for PropWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        if !area_state.prop_index_valid(self.prop_index) { return; }

        let prop = area_state.get_prop_mut(self.prop_index);

        prop.listeners.remove(NAME);
        if prop.is_active() {
            prop.toggle_active();
        }
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        if !area_state.prop_index_valid(self.prop_index) {
            // prop is invalid or has been removed.  close the window
            widget.borrow_mut().mark_for_removal();
            return Vec::new();
        }

        let prop = area_state.get_prop_mut(self.prop_index);

        prop.listeners.add(ChangeListener::invalidate(NAME, widget));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &prop.prop.name);

        let icon = Widget::with_theme(Label::empty(), "icon");
        icon.borrow_mut().state.foreground = Some(Rc::clone(&prop.prop.icon));

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let take_all = Widget::with_theme(Button::empty(), "take_all");

        let prop_index = self.prop_index;
        take_all.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            parent.borrow_mut().mark_for_removal();

            let pc = GameState::pc();
            let mut pc = pc.borrow_mut();
            pc.actor.take_all(prop_index);

            let root = Widget::get_root(&parent);
            let view = Widget::downcast_kind_mut::<RootView>(&root);
            view.set_inventory_window(&root, false);
        })));

        let list_content = Widget::empty("items_list");
        for (index, &(qty, ref item)) in prop.items().iter().enumerate() {
            let item_button = ItemButton::new(Some(item.item.icon.id()), qty,
                Some(index), Some(self.prop_index));
            let button = Widget::with_defaults(item_button);

            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
                let pc = GameState::pc();
                let mut pc = pc.borrow_mut();
                pc.actor.take(prop_index, index);
            })));
            Widget::add_child_to(&list_content, button);
        }

        vec![title, icon, close, list_content, take_all]
    }
}
