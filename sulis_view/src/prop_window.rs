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

use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use {item_button::take_item_cb, ItemButton, RootView};

pub const NAME: &str = "prop_window";

pub struct PropWindow {
    prop_index: usize,
    player: Rc<RefCell<EntityState>>,
}

impl PropWindow {
    pub fn new(prop_index: usize, player: Rc<RefCell<EntityState>>) -> Rc<RefCell<PropWindow>> {
        Rc::new(RefCell::new(PropWindow {
            prop_index,
            player,
        }))
    }

    pub fn prop_index(&self) -> usize { self.prop_index }

    pub fn player(&self) -> &Rc<RefCell<EntityState>> { &self.player }
}

impl WidgetKind for PropWindow {
    widget_kind!(NAME);

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

        let player_ref = Rc::clone(&self.player);
        let prop_index = self.prop_index;
        take_all.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(&widget);
            parent.borrow_mut().mark_for_removal();

            player_ref.borrow_mut().actor.take_all(prop_index);

            let root = Widget::get_root(&parent);
            let view = Widget::downcast_kind_mut::<RootView>(&root);
            view.set_inventory_window(&root, false);
        })));

        let list_content = Widget::empty("items_list");
        match prop.items() {
            None => (),
            Some(ref items) => {
                for (index, &(qty, ref item)) in items.iter().enumerate() {
                    let item_button = ItemButton::prop(item.item.icon.id(), qty, index, self.prop_index);
                    item_button.borrow_mut().add_action("Take", take_item_cb(&self.player, prop_index, index));
                    let button = Widget::with_defaults(item_button);
                    Widget::add_child_to(&list_content, button);
                }
            }
        }

        vec![title, icon, close, list_content, take_all]
    }
}
