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
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::{item_list_pane::Filter, ItemListPane, RootView};
use sulis_core::ui::{Callback, Widget, WidgetKind, RcRfc};
use sulis_core::widgets::{Button, Label};
use sulis_state::{ChangeListener, EntityState, GameState};

pub const NAME: &str = "prop_window";

pub struct PropWindow {
    prop_index: usize,
    player: RcRfc<EntityState>,
    filter: Rc<Cell<Filter>>,
}

impl PropWindow {
    pub fn new(prop_index: usize, player: RcRfc<EntityState>) -> RcRfc<PropWindow> {
        Rc::new(RefCell::new(PropWindow {
            prop_index,
            player,
            filter: Rc::new(Cell::new(Filter::All)),
        }))
    }

    pub fn prop_index(&self) -> usize {
        self.prop_index
    }

    pub fn player(&self) -> &RcRfc<EntityState> {
        &self.player
    }
}

impl WidgetKind for PropWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self, _widget: &RcRfc<Widget>) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        if !area_state.props().index_valid(self.prop_index) {
            return;
        }

        let prop = area_state.props_mut().get_mut(self.prop_index);

        prop.listeners.remove(NAME);
        if prop.is_active() {
            prop.toggle_active();
        }
    }

    fn on_add(&mut self, widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let icon = Widget::with_theme(Label::empty(), "icon");
        let close = Widget::with_theme(Button::empty(), "close");
        let take_all = Widget::with_theme(Button::empty(), "take_all");

        {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();

            if !area_state.props().index_valid(self.prop_index) {
                // prop is invalid or has been removed.  close the window
                widget.borrow_mut().mark_for_removal();
                return Vec::new();
            }

            let prop = area_state.props_mut().get_mut(self.prop_index);

            prop.listeners.add(ChangeListener::invalidate(NAME, widget));

            icon.borrow_mut().state.foreground = Some(Rc::clone(&prop.prop.icon));

            close
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(|widget, _| {
                    let (parent, _) = Widget::parent::<PropWindow>(widget);
                    parent.borrow_mut().mark_for_removal();
                })));

            let prop_index = self.prop_index;
            take_all
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, _) = Widget::parent::<PropWindow>(widget);
                    parent.borrow_mut().mark_for_removal();

                    let stash = GameState::party_stash();
                    stash.borrow_mut().take_all(prop_index);

                    let (root, view) = Widget::parent_mut::<RootView>(&parent);
                    view.set_inventory_window(&root, false);
                })));
            take_all
                .borrow_mut()
                .state
                .set_enabled(!GameState::is_combat_active());
        }

        let item_list_pane = Widget::with_defaults(ItemListPane::new_prop(
            &self.player,
            self.prop_index,
            &self.filter,
        ));

        vec![icon, close, item_list_pane, take_all]
    }
}
