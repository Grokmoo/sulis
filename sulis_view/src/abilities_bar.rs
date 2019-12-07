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
use std::cmp;
use std::collections::HashSet;
use std::rc::Rc;

use sulis_core::io::event;
use sulis_core::ui::{animation_state, Callback, Widget, WidgetKind, WidgetState};
use sulis_core::util::{ExtInt, Size};
use sulis_core::widgets::{Button, Label, TextArea, ScrollPane, ScrollDirection};
use sulis_module::{
    ability::{AbilityGroup, Duration},
    actor::OwnedAbility,
    Ability, Class, Module,
};
use sulis_state::{ChangeListener, EntityState, GameState, Script};

pub const NAME: &str = "abilities_bar";

pub struct AbilitiesBar {
    entity: Rc<RefCell<EntityState>>,
    group_panes: Vec<Rc<RefCell<Widget>>>,
    collapsed_panes: Vec<Rc<RefCell<Widget>>>,
    max_collapsed: u32,
}

impl AbilitiesBar {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilitiesBar>> {
        Rc::new(RefCell::new(AbilitiesBar {
            entity,
            group_panes: Vec::new(),
            collapsed_panes: Vec::new(),
            max_collapsed: 3,
        }))
    }
}

impl WidgetKind for AbilitiesBar {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref count) = widget.theme.custom.get("max_collapsed") {
            self.max_collapsed = count.parse::<u32>().unwrap_or(3);
        }

        widget.do_self_layout();

        // need to set the custom sizing for each panel prior to doing their layouts
        for widget in self.group_panes.iter() {
            let pane = Widget::kind_mut::<GroupPane>(widget);
            pane.set_layout_size(&mut widget.borrow_mut());
        }

        widget.do_children_layout();
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        {
            let mut entity = self.entity.borrow_mut();
            entity
                .actor
                .listeners
                .add(ChangeListener::invalidate(NAME, widget));
        }

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(
            NAME,
            Box::new(move |entity| {
                let entity = match entity {
                    None => return,
                    Some(entity) => entity,
                };
                let bar = Widget::kind_mut::<AbilitiesBar>(&widget_ref);
                bar.entity = Rc::clone(entity);
                widget_ref.borrow_mut().invalidate_children();
            }),
        ));

        self.group_panes.clear();
        let mut children = Vec::new();
        let abilities = self.entity.borrow().actor.actor.abilities.clone();

        let collapsed_group_ids: HashSet<String> = self
            .entity
            .borrow()
            .collapsed_groups()
            .into_iter()
            .collect();

        let mut collapsed_groups = Vec::new();
        let mut groups = Vec::new();
        for (index, group_id) in Module::rules().ability_groups.iter().enumerate() {
            let stats = &self.entity.borrow().actor.stats;
            if !stats.uses_per_encounter(group_id).is_zero()
                || !stats.uses_per_day(group_id).is_zero()
            {
                if !collapsed_group_ids.contains(group_id) {
                    groups.push(AbilityGroup { index });
                } else {
                    collapsed_groups.push(AbilityGroup { index });
                }
            }
        }

        let scrollpane = ScrollPane::new(ScrollDirection::Horizontal);

        let collapse_enabled = (collapsed_groups.len() as u32) < self.max_collapsed;
        if collapsed_groups.len() > 0 {
            let all_collapsed = Widget::empty("collapsed_panes");
            for group in collapsed_groups {
                let pane = CollapsedGroupPane::new(group, &self.entity);
                let widget = Widget::with_defaults(pane);

                self.collapsed_panes.push(Rc::clone(&widget));
                Widget::add_child_to(&all_collapsed, widget);
            }
            scrollpane.borrow().add_to_content(all_collapsed);
        }

        for group in groups {
            let group_pane = GroupPane::new(group, &self.entity, &abilities, collapse_enabled);

            if group_pane.borrow().abilities.is_empty() {
                continue;
            }

            let group_widget = Widget::with_defaults(group_pane);
            self.group_panes.push(Rc::clone(&group_widget));

            scrollpane.borrow().add_to_content(group_widget);
        }

        children.push(Widget::with_theme(scrollpane, "groups_pane"));
        children
    }
}

struct CollapsedGroupPane {
    entity: Rc<RefCell<EntityState>>,
    group: String,
    description: Rc<RefCell<Widget>>,
}

impl CollapsedGroupPane {
    fn new(
        group: AbilityGroup,
        entity: &Rc<RefCell<EntityState>>,
    ) -> Rc<RefCell<CollapsedGroupPane>> {
        Rc::new(RefCell::new(CollapsedGroupPane {
            group: group.name(),
            entity: Rc::clone(entity),
            description: Widget::with_theme(TextArea::empty(), "description"),
        }))
    }
}

impl WidgetKind for CollapsedGroupPane {
    widget_kind!("collapsed_group_pane");

    fn layout(&mut self, widget: &mut Widget) {
        self.description.borrow_mut().state.add_text_arg(
            "current_uses_per_encounter",
            &self
                .entity
                .borrow()
                .actor
                .current_uses_per_encounter(&self.group)
                .to_string(),
        );

        self.description.borrow_mut().state.add_text_arg(
            "current_uses_per_day",
            &self
                .entity
                .borrow()
                .actor
                .current_uses_per_day(&self.group)
                .to_string(),
        );

        widget.do_base_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.description.borrow_mut().state.clear_text_args();

        let total_uses = self
            .entity
            .borrow()
            .actor
            .stats
            .uses_per_encounter(&self.group);
        if !total_uses.is_infinite() && !total_uses.is_zero() {
            self.description
                .borrow_mut()
                .state
                .add_text_arg("total_uses_per_encounter", &total_uses.to_string());
        }

        let total_uses = self.entity.borrow().actor.stats.uses_per_day(&self.group);
        if !total_uses.is_infinite() && !total_uses.is_zero() {
            self.description
                .borrow_mut()
                .state
                .add_text_arg("total_uses_per_day", &total_uses.to_string());
        }

        self.description
            .borrow_mut()
            .state
            .add_text_arg("group", &self.group);

        let change_size = Widget::with_theme(Button::empty(), "change_size");
        let group = self.group.clone();
        let entity = Rc::clone(&self.entity);
        change_size
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                entity.borrow_mut().remove_collapsed_group(&group);

                let (parent, _) = Widget::parent::<AbilitiesBar>(widget);
                parent.borrow_mut().invalidate_children();

                let root = Widget::get_root(widget);
                Widget::remove_mouse_over(&root);
            })));

        vec![self.description.clone(), change_size]
    }
}
struct GroupPane {
    entity: Rc<RefCell<EntityState>>,
    abilities: Vec<OwnedAbility>,
    group: String,
    description: Rc<RefCell<Widget>>,
    skip_first_position: bool,
    vertical_count: u32,
    min_horizontal_count: u32,
    grid_width: u32,
    grid_border: u32,
    collapse_enabled: bool,
}

impl GroupPane {
    fn new(
        group: AbilityGroup,
        entity: &Rc<RefCell<EntityState>>,
        abilities: &Vec<OwnedAbility>,
        collapse_enabled: bool,
    ) -> Rc<RefCell<GroupPane>> {
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
            skip_first_position: true,
            min_horizontal_count: 1,
            collapse_enabled,
        }))
    }

    fn set_layout_size(&mut self, widget: &mut Widget) {
        let theme = &widget.theme;

        self.vertical_count = theme.get_custom_or_default("vertical_count", 1);
        self.grid_width = theme.get_custom_or_default("grid_width", 1);
        self.min_horizontal_count = theme.get_custom_or_default("min_horizontal_count", 1);
        self.grid_border = theme.get_custom_or_default("grid_border", 1);
        self.skip_first_position = theme.get_custom_or_default("skip_first_position", false);

        let height = widget.state.height();

        let mut items_count = self.abilities.len();
        if self.skip_first_position { items_count += 1; }

        let cols = (items_count as f32 / self.vertical_count as f32).ceil() as u32;
        let cols = cmp::max(cols, self.min_horizontal_count) as i32;
        let width = cols * self.grid_width as i32 + self.grid_border as i32;
        widget.state.set_size(Size::new(width, height));
    }
}

impl WidgetKind for GroupPane {
    widget_kind!("group_pane");

    fn layout(&mut self, widget: &mut Widget) {
        self.description.borrow_mut().state.add_text_arg(
            "current_uses_per_encounter",
            &self
                .entity
                .borrow()
                .actor
                .current_uses_per_encounter(&self.group)
                .to_string(),
        );

        self.description.borrow_mut().state.add_text_arg(
            "current_uses_per_day",
            &self
                .entity
                .borrow()
                .actor
                .current_uses_per_day(&self.group)
                .to_string(),
        );

        widget.do_base_layout();
    }

    fn on_remove(&mut self, _widget: &Rc<RefCell<Widget>>) {
        for ability in self.abilities.iter() {
            if let Some(ref mut state) = self
                .entity
                .borrow_mut()
                .actor
                .ability_state(&ability.ability.id)
            {
                state.listeners.remove(&ability.ability.id);
            }
        }
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        for ability in self.abilities.iter() {
            if let Some(ref mut state) = self
                .entity
                .borrow_mut()
                .actor
                .ability_state(&ability.ability.id)
            {
                state.listeners.add(ChangeListener::invalidate_layout(
                    &ability.ability.id,
                    widget,
                ));
            }
        }

        self.description.borrow_mut().state.clear_text_args();

        let total_uses = self
            .entity
            .borrow()
            .actor
            .stats
            .uses_per_encounter(&self.group);
        if !total_uses.is_infinite() && !total_uses.is_zero() {
            self.description
                .borrow_mut()
                .state
                .add_text_arg("total_uses_per_encounter", &total_uses.to_string());
        }

        let total_uses = self.entity.borrow().actor.stats.uses_per_day(&self.group);
        if !total_uses.is_infinite() && !total_uses.is_zero() {
            self.description
                .borrow_mut()
                .state
                .add_text_arg("total_uses_per_day", &total_uses.to_string());
        }

        self.description
            .borrow_mut()
            .state
            .add_text_arg("group", &self.group);

        let abilities_pane = Widget::empty("abilities_pane");

        if self.skip_first_position {
            let skip = Widget::empty("skip_button");
            Widget::add_child_to(&abilities_pane, skip);
        }

        for ability in self.abilities.iter() {
            let ability = &ability.ability;

            let button = Widget::with_defaults(AbilityButton::new(ability, &self.entity));
            Widget::add_child_to(&abilities_pane, button);
        }

        let change_size = Widget::with_theme(Button::empty(), "change_size");
        change_size
            .borrow_mut()
            .state
            .set_enabled(self.collapse_enabled);
        let group = self.group.clone();
        let entity = Rc::clone(&self.entity);
        change_size
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                entity.borrow_mut().add_collapsed_group(group.clone());

                let (parent, _) = Widget::parent::<AbilitiesBar>(widget);
                parent.borrow_mut().invalidate_children();

                let root = Widget::get_root(widget);
                Widget::remove_mouse_over(&root);
            })));

        vec![self.description.clone(), abilities_pane, change_size]
    }
}

struct AbilityButton {
    entity: Rc<RefCell<EntityState>>,
    ability: Rc<Ability>,
    newly_added: bool,
}

impl AbilityButton {
    fn new(ability: &Rc<Ability>, entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<AbilityButton>> {
        let mut newly_added = false;
        if let Some(state) = entity.borrow_mut().actor.ability_state(&ability.id) {
            newly_added = state.newly_added_ability;
            state.newly_added_ability = false;
        }

        Rc::new(RefCell::new(AbilityButton {
            ability: Rc::clone(ability),
            entity: Rc::clone(entity),
            newly_added,
        }))
    }
}

impl WidgetKind for AbilityButton {
    widget_kind!("ability_button");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if self.entity.borrow().actor.can_toggle(&self.ability.id) {
            widget
                .state
                .animation_state
                .remove(animation_state::Kind::Custom1);
        } else {
            widget
                .state
                .animation_state
                .add(animation_state::Kind::Custom1);
        }

        if let Some(ref mut state) = self
            .entity
            .borrow_mut()
            .actor
            .ability_state(&self.ability.id)
        {
            if self.newly_added {
                widget.state.animation_state.add(animation_state::Kind::Custom2);
            } else {
                widget.state.animation_state.remove(animation_state::Kind::Custom2);
            }

            widget.children[1].borrow_mut().state.clear_text_args();
            let child = &mut widget.children[1].borrow_mut().state;
            match state.remaining_duration_rounds() {
                ExtInt::Infinity => child.add_text_arg("duration", "Active"),
                ExtInt::Int(rounds) => {
                    if rounds != 0 {
                        child.add_text_arg("duration", &rounds.to_string());
                    }
                }
            }
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let duration_label = Widget::with_theme(Label::empty(), "duration_label");
        duration_label.borrow_mut().state.set_enabled(false);
        let icon = Widget::empty("icon");
        icon.borrow_mut()
            .state
            .add_text_arg("icon", &self.ability.icon.id());
        icon.borrow_mut().state.set_enabled(false);

        vec![icon, duration_label]
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        let hover = Widget::with_theme(TextArea::empty(), "ability_hover");
        let class = self.entity.borrow_mut().actor.actor.base_class();
        add_hover_text_args(&mut hover.borrow_mut().state, &self.ability, &class);

        if self.newly_added {
            hover.borrow_mut().state.add_text_arg("newly_added", "true");
        }

        Widget::set_mouse_over_widget(
            &widget,
            hover,
            widget.borrow().state.inner_right(),
            widget.borrow().state.inner_top(),
        );

        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        let can_activate = self.entity.borrow().actor.can_activate(&self.ability.id);
        if can_activate {
            let index = self.entity.borrow().index();
            Script::ability_on_activate(index, &self.ability);
            return true;
        }

        let can_toggle = self.entity.borrow().actor.can_toggle(&self.ability.id);
        if can_toggle {
            let index = self.entity.borrow().index();
            Script::ability_on_deactivate(index, &self.ability);
        }

        true
    }
}

pub fn add_hover_text_args(state: &mut WidgetState, ability: &Ability, class: &Class) {
    state.disable();
    state.add_text_arg("name", &ability.name);
    state.add_text_arg("description", &ability.description);

    for (index, upgrade) in ability.upgrades.iter().enumerate() {
        state.add_text_arg(&format!("upgrade{}", index + 1), &upgrade.description);
    }

    if let Some(ref active) = ability.active {
        let ap = Module::rules().to_display_ap(active.ap as i32);
        state.add_text_arg("activate_ap", &ap.to_string());

        if let Some(class_stats) = active.class_stats.get(&class.id) {
            for stat in &class.stats {
                if !stat.display {
                    continue;
                }
                if let Some(amount) = class_stats.get(&stat.id) {
                    state.add_text_arg("class_stat_name", &stat.name);
                    state.add_text_arg("class_stat_amount", &amount.to_string());
                }
            }
        }

        match active.duration {
            Duration::Rounds(rounds) => state.add_text_arg("duration", &rounds.to_string()),
            Duration::Mode => state.add_text_arg("mode", "true"),
            Duration::Instant => state.add_text_arg("instant", "true"),
            Duration::Permanent => state.add_text_arg("permanent", "true"),
        }

        if active.cooldown != 0 {
            state.add_text_arg("cooldown", &active.cooldown.to_string());
        }

        state.add_text_arg("short_description", &active.short_description);
    }
}
