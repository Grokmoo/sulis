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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use sulis_module::{Ability};

use {AbilityPane, CharacterBuilder};
use character_builder::BuilderPane;

pub const NAME: &str = "ability_selector_pane";

pub struct AbilitySelectorPane {
    choices: Vec<Rc<Ability>>,
    selected_ability: Option<Rc<Ability>>,
    index: usize,
}

impl AbilitySelectorPane {
    pub fn new(choices: Vec<Rc<Ability>>, index: usize) -> Rc<RefCell<AbilitySelectorPane>> {
        Rc::new(RefCell::new(AbilitySelectorPane {
            selected_ability: None,
            index,
            choices,
        }))
    }
}

impl BuilderPane for AbilitySelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, _widget: Rc<RefCell<Widget>>) {
        // remove any abilities selected by this pane and subsequent ability panes
        builder.abilities.truncate(self.index);
        builder.prev.borrow_mut().state.set_enabled(true);
        builder.next.borrow_mut().state.set_enabled(self.selected_ability.is_some());
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let ability = match self.selected_ability {
            None => return,
            Some(ref ability) => Rc::clone(ability),
        };
        builder.abilities.push(ability);
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.prev(&widget);
    }
}

impl WidgetKind for AbilitySelectorPane {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let abilities_pane = Widget::empty("abilities_pane");
        for ability in self.choices.iter() {
            let ability_button = Widget::with_theme(Button::empty(), "ability_button");
            ability_button.borrow_mut().state.add_text_arg("icon", &ability.icon.id());
            if let Some(ref selected_ability) = self.selected_ability {
                ability_button.borrow_mut().state.set_active(*ability == *selected_ability);
            }

            let ability_ref = Rc::clone(&ability);
            ability_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(&widget, 2);
                let pane = Widget::downcast_kind_mut::<AbilitySelectorPane>(&parent);
                pane.selected_ability = Some(Rc::clone(&ability_ref));
                parent.borrow_mut().invalidate_children();

                let builder_widget = Widget::get_parent(&parent);
                let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);
                builder.next.borrow_mut().state.set_enabled(true);
            })));

            Widget::add_child_to(&abilities_pane, ability_button);
        }

        let ability = match self.selected_ability {
            None => return vec![title, abilities_pane],
            Some(ref ability) => ability,
        };

        let ability_pane = AbilityPane::empty();
        ability_pane.borrow_mut().set_ability(Rc::clone(ability));
        let ability_pane_widget = Widget::with_defaults(ability_pane);

        vec![title, ability_pane_widget, abilities_pane]
    }
}
