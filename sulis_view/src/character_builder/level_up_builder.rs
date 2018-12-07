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

use sulis_core::ui::{Widget};
use sulis_module::Actor;
use sulis_state::EntityState;

use crate::character_builder::*;

pub struct LevelUpBuilder {
    pub pc: Rc<RefCell<EntityState>>,
}

impl BuilderSet for LevelUpBuilder {
    fn on_add(&self, builder: &mut CharacterBuilder,
              _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut children = Vec::new();

        let choices = vec![self.pc.borrow().actor.actor.base_class().id.to_string()];
        let class_selector_pane = ClassSelectorPane::new(choices, false);
        let level_up_finish_pane = LevelUpFinishPane::new();
        let class_sel_widget = Widget::with_defaults(class_selector_pane.clone());
        let level_up_finish_widget = Widget::with_defaults(level_up_finish_pane.clone());

        let class_sel_widget_ref = Rc::clone(&class_sel_widget);
        let class_selector_pane_ref = Rc::clone(&class_selector_pane);

        class_sel_widget.borrow_mut().state.set_visible(true);
        level_up_finish_widget.borrow_mut().state.set_visible(false);
        builder.finish.borrow_mut().state.set_visible(false);

        builder.builder_panes.clear();
        builder.builder_pane_index = 0;
        builder.builder_panes.push(class_selector_pane);
        children.push(class_sel_widget);

        let actor = &self.pc.borrow().actor.actor;
        let level = actor.total_level + 1;
        for (index, ability_list) in actor.base_class().ability_choices(level).into_iter().enumerate() {
            let pane = AbilitySelectorPane::new(ability_list, index, Rc::clone(&self.pc),
                actor.abilities.clone());
            let widget = Widget::with_defaults(pane.clone());
            widget.borrow_mut().state.set_visible(false);

            builder.builder_panes.push(pane);
            children.push(widget);
        }

        builder.builder_panes.push(level_up_finish_pane.clone());
        children.push(level_up_finish_widget);

        class_selector_pane_ref.borrow_mut().on_selected(builder, Rc::clone(&class_sel_widget_ref));
        children
    }

    fn finish(&self, builder: &mut CharacterBuilder, _widget: &Rc<RefCell<Widget>>) {
        let class = match builder.class {
            None => return,
            Some(ref class) => Rc::clone(class),
        };

        let mut pc = self.pc.borrow_mut();
        let state = &mut pc.actor;

        let new_actor = Actor::from(&state.actor, Some((class, 1)), state.xp(),
            builder.abilities.clone(), Vec::new(), state.actor.inventory.clone());
        state.replace_actor(new_actor);
        state.init_day();
    }
}
