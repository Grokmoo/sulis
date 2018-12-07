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

use sulis_core::ui::{animation_state, Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label};
use sulis_module::{Ability, AbilityList, Actor, actor::OwnedAbility};
use sulis_state::EntityState;

use crate::{AbilityPane, CharacterBuilder};
use crate::character_builder::BuilderPane;

pub const NAME: &str = "ability_selector_pane";

pub struct AbilitySelectorPane {
    already_selected: Vec<OwnedAbility>,
    prereqs_not_met: HashSet<Rc<Ability>>,
    choices: Rc<AbilityList>,
    selected_ability: Option<Rc<Ability>>,
    index: usize,
    pc: Rc<RefCell<EntityState>>,
}

impl AbilitySelectorPane {
    pub fn new(choices: Rc<AbilityList>, index: usize, pc: Rc<RefCell<EntityState>>,
               already_selected: Vec<OwnedAbility>) -> Rc<RefCell<AbilitySelectorPane>> {
        Rc::new(RefCell::new(AbilitySelectorPane {
            selected_ability: None,
            index,
            choices,
            already_selected,
            prereqs_not_met: HashSet::new(),
            pc,
        }))
    }

    pub fn add_already_selected(&mut self, ability: &Rc<Ability>) {
        for owned in self.already_selected.iter_mut() {
            if Rc::ptr_eq(&owned.ability, ability) {
                owned.level += 1;
                return;
            }
        }

        self.already_selected.push(OwnedAbility { ability: Rc::clone(ability), level: 0 });
    }

    pub fn remove_already_selected(&mut self, ability: &Rc<Ability>) {
        let mut index_to_remove = 0;
        for (index, owned) in self.already_selected.iter_mut().enumerate() {
            if Rc::ptr_eq(&owned.ability, ability) {
                if owned.level == 0 {
                    index_to_remove = index;
                    break;
                }

                owned.level -= 1;
                return;
            }
        }

        self.already_selected.remove(index_to_remove);
    }

    pub fn already_selected_current_level(&self, ability: &Rc<Ability>) -> Option<u32> {
        for owned in self.already_selected.iter() {
            if Rc::ptr_eq(&ability, &owned.ability) {
                return Some(owned.level);
            }
        }
        None
    }
}

impl BuilderPane for AbilitySelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        // remove any abilities selected by this pane and subsequent ability panes
        builder.abilities.truncate(self.index);
        builder.prev.borrow_mut().state.set_enabled(true);

        for ability in builder.abilities.iter() {
            self.add_already_selected(ability);
        }

        let prereq_actor = {
            let state = &self.pc.borrow().actor;

            let class = match builder.class.clone() {
                Some(class) => Some((class, 1)),
                None => None,
            };

            Rc::new(Actor::from(&state.actor, class, 0,
                builder.abilities.clone(), Vec::new(), state.actor.inventory.clone()))
        };

        self.prereqs_not_met.clear();
        for entry in self.choices.iter() {
            let ability = &entry.ability;
            if !ability.meets_prereqs(&prereq_actor) {
                self.prereqs_not_met.insert(Rc::clone(ability));
            }
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
            self.remove_already_selected(ability);
        }
        builder.prev(&widget);
    }
}

struct AbilitiesPane {
    positions: Vec<(f32, f32)>,
    grid_size: i32,
    grid_border: i32,
    id: String,
}

impl AbilitiesPane {
    fn new(id: &str) -> AbilitiesPane {
        AbilitiesPane {
            positions: Vec::new(),
            grid_size: 10,
            grid_border: 1,
            id: format!("abilities_pane_{}", id),
        }
    }
}

impl WidgetKind for AbilitiesPane {
    fn get_name(&self) -> &str { &self.id }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref theme) = widget.theme {
            self.grid_size = theme.get_custom_or_default("grid_size", 10);
            self.grid_border = theme.get_custom_or_default("grid_border", 1);
        }

        let grid = self.grid_size as f32;
        widget.do_self_layout();
        for (index, child) in widget.children.iter().enumerate() {
            let position = self.positions[index];
            let pos_x = (position.0 * grid) as i32 + self.grid_border;
            let pos_y = (position.1 * grid) as i32 + self.grid_border;
            child.borrow_mut().state.set_position(widget.state.inner_position.x + pos_x,
                                                  widget.state.inner_position.y + pos_y);
        }

        widget.do_children_layout();
    }
}

impl WidgetKind for AbilitySelectorPane {
    widget_kind![NAME];

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let pane = Rc::new(RefCell::new(AbilitiesPane::new(&self.choices.id)));
        let abilities_pane = Widget::with_defaults(pane.clone());
        for entry in self.choices.iter() {
            let ability = &entry.ability;
            let position = entry.position;
            pane.borrow_mut().positions.push(position);

            let ability_button = Widget::with_theme(Button::empty(), "ability_button");

            let level = self.already_selected_current_level(ability);

            if self.prereqs_not_met.contains(ability) {
                ability_button.borrow_mut().state.animation_state.add(animation_state::Kind::Custom2);
            }

            let icon = Widget::with_theme(Label::empty(), "icon");
            icon.borrow_mut().state.add_text_arg("icon", &ability.icon.id());
            Widget::add_child_to(&ability_button, icon);

            let upgrade = Widget::with_theme(Label::empty(), "upgrade0");
            if let Some(_) = level {
                upgrade.borrow_mut().state.set_active(true);
            }
            Widget::add_child_to(&ability_button, upgrade);

            for (index, _) in ability.upgrades.iter().enumerate() {
                let upgrade = Widget::with_theme(Label::empty(), &format!("upgrade{}", index+1));
                if let Some(level) = level {
                    if level as usize > index {
                        upgrade.borrow_mut().state.set_active(true);
                    }
                }
                Widget::add_child_to(&ability_button, upgrade);
            }

            if let Some(ref selected_ability) = self.selected_ability {
                ability_button.borrow_mut().state.set_active(*ability == *selected_ability);
            }

            let mut enable_next = true;
            if let Some(level) = level {
                if level as usize == ability.upgrades.len() { enable_next = false; }
            }
            if self.prereqs_not_met.contains(ability) { enable_next = false; }

            let ability_ref = Rc::clone(&ability);
            ability_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(&widget, 2);
                let pane = Widget::downcast_kind_mut::<AbilitySelectorPane>(&parent);
                pane.selected_ability = Some(Rc::clone(&ability_ref));
                parent.borrow_mut().invalidate_children();

                let builder_widget = Widget::get_parent(&parent);
                let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);
                builder.next.borrow_mut().state.set_enabled(enable_next);
            })));

            Widget::add_child_to(&abilities_pane, ability_button);
        }

        let ability = match self.selected_ability {
            None => return vec![title, abilities_pane],
            Some(ref ability) => ability,
        };

        let ability_pane = AbilityPane::empty();
        ability_pane.borrow_mut().set_ability(Rc::clone(ability));
        let ability_pane_widget = Widget::with_defaults(ability_pane.clone());

        let details = &ability_pane.borrow().details;
        if self.prereqs_not_met.contains(ability) {
            details.borrow_mut().state.add_text_arg("prereqs_not_met", "true");
        }

        if let Some(level) = self.already_selected_current_level(ability) {
            details.borrow_mut().state.add_text_arg("owned_level", &(level + 1).to_string());
        }

        vec![title, ability_pane_widget, abilities_pane]
    }
}
