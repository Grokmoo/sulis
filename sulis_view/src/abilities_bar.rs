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

use std::cmp;
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::io::event;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::util::{Size, ExtInt};
use sulis_module::{actor::OwnedAbility, Ability, Module, ability::{AbilityGroup, Duration}};
use sulis_state::{ChangeListener, EntityState, GameState};
use sulis_widgets::{Label, TextArea};

pub const NAME: &str = "abilities_bar";

pub struct AbilitiesBar {
    entity: Rc<RefCell<EntityState>>,
    group_panes: Vec<Rc<RefCell<Widget>>>,
}

impl AbilitiesBar {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilitiesBar>> {
        Rc::new(RefCell::new(AbilitiesBar {
            entity,
            group_panes: Vec::new(),
        }))
    }
}

impl WidgetKind for AbilitiesBar {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_self_layout();

        // need to set the custom sizing for each panel prior to doing their layouts
        for widget in self.group_panes.iter() {
            let pane = Widget::downcast_kind_mut::<GroupPane>(widget);
            pane.set_layout_size(&mut widget.borrow_mut());
        }

        widget.do_children_layout();
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        {
            let mut entity = self.entity.borrow_mut();
            entity.actor.listeners.add(ChangeListener::invalidate(NAME, widget));
        }

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |entity| {
            let entity = match entity {
                None => return,
                Some(entity) => entity,
            };
            let bar = Widget::downcast_kind_mut::<AbilitiesBar>(&widget_ref);
            bar.entity = Rc::clone(entity);
            widget_ref.borrow_mut().invalidate_children();
        })));

        self.group_panes.clear();
        let mut children = Vec::new();
        let abilities = self.entity.borrow().actor.actor.abilities.clone();

        let mut groups = Vec::new();
        for (index, group_id) in Module::rules().ability_groups.iter().enumerate() {
            if !self.entity.borrow().actor.stats.uses_per_encounter(group_id).is_zero() {
                groups.push(AbilityGroup { index });
            }
        }

        for group in groups {
            let group_pane = GroupPane::new(group, &self.entity, &abilities);

            if group_pane.borrow().abilities.is_empty() { continue; }

            let group_widget = Widget::with_defaults(group_pane);
            self.group_panes.push(Rc::clone(&group_widget));
            children.push(group_widget);
        }

        children
    }
}

struct GroupPane {
    entity: Rc<RefCell<EntityState>>,
    abilities: Vec<OwnedAbility>,
    group: String,
    description: Rc<RefCell<Widget>>,
    vertical_count: u32,
    min_horizontal_count: u32,
    grid_width: u32,
    grid_border: u32,
}

impl GroupPane {
    fn new(group: AbilityGroup, entity: &Rc<RefCell<EntityState>>,
           abilities: &Vec<OwnedAbility>) -> Rc<RefCell<GroupPane>> {
        let mut abilities_to_add = Vec::new();
        for ability in abilities.iter() {
            let active = match ability.ability.active {
                None => continue,
                Some(ref active) => active,
            };

            if active.group == group {
                abilities_to_add.push(ability.clone());
            }
        }
        Rc::new(RefCell::new(GroupPane {
            group: group.name(),
            entity: Rc::clone(entity),
            abilities: abilities_to_add,
            description: Widget::with_theme(TextArea::empty(), "description"),
            grid_width: 1,
            grid_border: 1,
            vertical_count: 1,
            min_horizontal_count: 1,
        }))
    }

    fn set_layout_size(&mut self, widget: &mut Widget) {
        if let Some(ref theme) = widget.theme {
            if let Some(ref count) = theme.custom.get("vertical_count") {
                self.vertical_count = count.parse::<u32>().unwrap_or(1);
            }

            if let Some(ref width) = theme.custom.get("grid_width") {
                self.grid_width = width.parse::<u32>().unwrap_or(1);
            }

            if let Some(ref count) = theme.custom.get("min_horizontal_count") {
                self.min_horizontal_count = count.parse::<u32>().unwrap_or(1);
            }

            if let Some(ref width) = theme.custom.get("grid_border") {
                self.grid_border = width.parse::<u32>().unwrap_or(1);
            }
        }

        let height = widget.state.size.height;
        let cols = (self.abilities.len() as f32 / self.vertical_count as f32).ceil() as u32;
        let cols = cmp::max(cols, self.min_horizontal_count) as i32;
        let width = cols * self.grid_width as i32 + self.grid_border as i32;
        widget.state.set_size(Size::new(width, height));
    }
}

impl WidgetKind for GroupPane {
    widget_kind!("group_pane");

    fn layout(&mut self, widget: &mut Widget) {
        self.description.borrow_mut().state.add_text_arg("current_uses",
            &self.entity.borrow().actor.current_uses_per_encounter(&self.group).to_string());

        widget.do_base_layout();
    }

    fn on_remove(&mut self) {
        for ability in self.abilities.iter() {
            if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&ability.ability.id) {
                state.listeners.remove(&ability.ability.id);
            }
        }
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        for ability in self.abilities.iter() {
            if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&ability.ability.id) {
                state.listeners.add(ChangeListener::invalidate_layout(&ability.ability.id, widget));
            }
        }

        self.description.borrow_mut().state.clear_text_args();

        let total_uses = self.entity.borrow().actor.stats.uses_per_encounter(&self.group);
        if !total_uses.is_infinite() {
            self.description.borrow_mut().state.add_text_arg("total_uses", &total_uses.to_string());
        }

        self.description.borrow_mut().state.add_text_arg("group", &self.group);

        let abilities_pane = Widget::empty("abilities_pane");
        for ability in self.abilities.iter() {
            let ability = &ability.ability;

            let button = Widget::with_defaults(AbilityButton::new(ability, &self.entity));
            Widget::add_child_to(&abilities_pane, button);
        }

        vec![self.description.clone(), abilities_pane]
    }
}

struct AbilityButton {
    entity: Rc<RefCell<EntityState>>,
    ability: Rc<Ability>,
}

impl AbilityButton {
    fn new(ability: &Rc<Ability>, entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilityButton>> {
        Rc::new(RefCell::new(AbilityButton {
            ability: Rc::clone(ability),
            entity: Rc::clone(entity),
        }))
    }
}

impl WidgetKind for AbilityButton {
    widget_kind!("ability_button");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        widget.state.set_enabled(self.entity.borrow().actor.can_toggle(&self.ability.id));

        if let Some(ref mut state) = self.entity.borrow_mut().actor.ability_state(&self.ability.id) {
            widget.children[1].borrow_mut().state.clear_text_args();
            let mut child = &mut widget.children[1].borrow_mut().state;
            match state.remaining_duration_rounds() {
                ExtInt::Infinity => child.add_text_arg("duration", "Active"),
                ExtInt::Int(rounds) => {
                    if rounds != 0 { child.add_text_arg("duration", &rounds.to_string()); }
                }
            }
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        let duration_label = Widget::with_theme(Label::empty(), "duration_label");
        duration_label.borrow_mut().state.set_enabled(false);
        let icon = Widget::empty("icon");
        icon.borrow_mut().state.add_text_arg("icon", &self.ability.icon.id());
        icon.borrow_mut().state.set_enabled(false);

        vec![icon, duration_label]
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        let hover = Widget::with_theme(TextArea::empty(), "ability_hover");
        {
            let mut hover = hover.borrow_mut();
            hover.state.disable();

            hover.state.add_text_arg("name", &self.ability.name);
            hover.state.add_text_arg("description", &self.ability.description);
            if let Some(ref active) = self.ability.active {
                let ap = active.ap / Module::rules().display_ap;
                hover.state.add_text_arg("activate_ap", &ap.to_string());

                match active.duration {
                    Duration::Rounds(rounds) => hover.state.add_text_arg("duration", &rounds.to_string()),
                    Duration::Mode => hover.state.add_text_arg("mode", "true"),
                    Duration::Instant => hover.state.add_text_arg("instant", "true"),
                    Duration::Permanent => hover.state.add_text_arg("permanent", "true"),
                }

                if active.cooldown != 0 {
                    hover.state.add_text_arg("cooldown", &active.cooldown.to_string());
                }

                hover.state.add_text_arg("short_description", &active.short_description);
            }
        }

        Widget::set_mouse_over_widget(&widget, hover, widget.borrow().state.inner_right(),
            widget.borrow().state.inner_top());

        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        if self.entity.borrow().actor.can_activate(&self.ability.id) {
            GameState::execute_ability_on_activate(&self.entity, &self.ability);
        } else if self.entity.borrow().actor.can_toggle(&self.ability.id) {
            self.entity.borrow_mut().actor.deactivate_ability_state(&self.ability.id);
        }

        true
    }
}
