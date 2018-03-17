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
use std::collections::HashSet;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_widgets::{Button, Label};
use sulis_module::{Ability, AbilityList};

use {AbilityPane, CharacterBuilder};
use character_builder::BuilderPane;

pub const NAME: &str = "ability_selector_pane";

pub struct AbilitySelectorPane {
    already_selected: HashSet<Rc<Ability>>,
    choices: Rc<AbilityList>,
    selected_ability: Option<Rc<Ability>>,
    index: usize,
}

impl AbilitySelectorPane {
    pub fn new(choices: Rc<AbilityList>, index: usize,
               already_selected: Vec<Rc<Ability>>) -> Rc<RefCell<AbilitySelectorPane>> {
        Rc::new(RefCell::new(AbilitySelectorPane {
            selected_ability: None,
            index,
            choices,
            already_selected: already_selected.into_iter().collect(),
        }))
    }
}

impl BuilderPane for AbilitySelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        // remove any abilities selected by this pane and subsequent ability panes
        builder.abilities.truncate(self.index);
        builder.prev.borrow_mut().state.set_enabled(true);

        for ability in builder.abilities.iter() {
            self.already_selected.insert(Rc::clone(ability));
        }

        widget.borrow_mut().invalidate_children();

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
        self.selected_ability = None;
        for ability in builder.abilities.iter() {
            self.already_selected.remove(ability);
        }
        builder.prev(&widget);
    }
}

struct AbilitiesPane {
    positions: Vec<Point>,
}

impl WidgetKind for AbilitiesPane {
    widget_kind!["abilities_pane"];

    fn layout(&mut self, widget: &mut Widget) {
        let mut grid_size = 10;
        if let Some(ref theme) = widget.theme {
            grid_size = theme.get_custom_or_default("grid_size", 10);
        }

        widget.do_self_layout();
        for (index, child) in widget.children.iter().enumerate() {
            let position = self.positions[index];
            child.borrow_mut().state.set_position(widget.state.inner_position.x + position.x * grid_size,
                                                  widget.state.inner_position.y + position.y * grid_size);
        }

        widget.do_children_layout();
    }
}

impl WidgetKind for AbilitySelectorPane {
    widget_kind![NAME];

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let pane = Rc::new(RefCell::new(AbilitiesPane { positions: Vec::new() }));
        let abilities_pane = Widget::with_defaults(pane.clone());
        for entry in self.choices.iter() {
            let ability = &entry.ability;
            let position = entry.position;
            pane.borrow_mut().positions.push(position);

            let mut already_selected = false;
            for already_ability in self.already_selected.iter() {
                if already_ability == ability {
                    already_selected = true;
                }
            }

            let ability_button = Widget::with_theme(Button::empty(), "ability_button");
            if already_selected {
                ability_button.borrow_mut().state.set_enabled(false);
            }

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
