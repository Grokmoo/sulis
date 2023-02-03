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

use std::collections::HashMap;

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label, Spinner, TextArea};
use sulis_module::{Ability, Attribute, AttributeList, BonusKind, Class, Module, Race};

use crate::abilities_bar::add_hover_text_args;
use crate::character_builder::BuilderPane;
use crate::CharacterBuilder;

pub const NAME: &str = "attribute_selector_pane";

pub struct AttributeSelectorPane {
    available: i32,
    attrs: AttributeList,
    selected_race: Option<Rc<Race>>,
    selected_class: Option<Rc<Class>>,
    selected_kit: Option<usize>,
}

impl AttributeSelectorPane {
    pub fn new() -> Rc<RefCell<AttributeSelectorPane>> {
        let rules = Module::rules();
        let attrs = AttributeList::new(rules.base_attribute as u8);

        let total = rules.base_attribute * (Attribute::iter().count() as i32);
        let available = rules.builder_attribute_points - total;

        Rc::new(RefCell::new(AttributeSelectorPane {
            attrs,
            available,
            selected_race: None,
            selected_class: None,
            selected_kit: None,
        }))
    }

    fn calculate_available(&mut self) {
        let rules = Module::rules();

        let mut total = 0;
        for attr in Attribute::iter() {
            total += self.attrs.get(*attr) as i32;
        }
        self.available = rules.builder_attribute_points - total;
    }

    fn set_next_enabled(&mut self, widget: &Rc<RefCell<Widget>>) {
        self.calculate_available();

        let (_, builder) = Widget::parent_mut::<CharacterBuilder>(widget);
        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.available == 0 && self.selected_kit.is_some());
    }
}

impl BuilderPane for AttributeSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        self.selected_class = builder.class.clone();
        self.selected_race = builder.race.clone();

        if let Some(ref class) = self.selected_class {
            self.selected_kit = Some(0);
            self.attrs = class.kits[0].default_attributes;
        }

        builder.attributes = None;
        builder.kit = None;
        builder.prev.borrow_mut().state.set_enabled(true);
        self.calculate_available();
        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.available == 0 && self.selected_kit.is_some());
        widget.borrow_mut().invalidate_children();
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let class = match &self.selected_class {
            None => return,
            Some(ref class) => class,
        };

        let kit = match &self.selected_kit {
            None => return,
            Some(index) => &class.kits[*index],
        };

        builder.kit = self.selected_kit;
        builder.attributes = Some(self.attrs);
        builder.inventory = Some(kit.starting_inventory.clone());
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let rules = Module::rules();
        self.attrs = AttributeList::new(rules.base_attribute as u8);
        self.selected_kit = None;
        builder.prev(&widget);
    }
}

impl WidgetKind for AttributeSelectorPane {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let rules = Module::rules();
        let mut children = Vec::new();

        let title = Widget::with_theme(Label::empty(), "title");
        children.push(title);

        let class = match &self.selected_class {
            None => return children,
            Some(ref class) => class,
        };

        let race = match &self.selected_race {
            None => return children,
            Some(ref race) => race,
        };

        let kits_pane = Widget::empty("kits_pane");
        for (index, kit) in class.kits.iter().enumerate() {
            let kit_button = Widget::with_theme(Button::empty(), "kit_button");
            kit_button
                .borrow_mut()
                .state
                .add_text_arg("name", &kit.name);
            if let Some(selected_index) = self.selected_kit {
                kit_button
                    .borrow_mut()
                    .state
                    .set_active(selected_index == index);
            }
            let class_ref = Rc::clone(class);
            kit_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, pane) = Widget::parent_mut::<AttributeSelectorPane>(widget);
                    pane.selected_kit = Some(index);
                    pane.attrs = class_ref.kits[index].default_attributes;
                    pane.set_next_enabled(&parent);

                    parent.borrow_mut().invalidate_children();
                })));
            Widget::add_child_to(&kits_pane, kit_button);
        }
        children.push(kits_pane);

        let selected_kit = match &self.selected_kit {
            None => return children,
            Some(index) => &class.kits[*index],
        };

        let kit_area = Widget::with_theme(TextArea::empty(), "kit_area");
        kit_area
            .borrow_mut()
            .state
            .add_text_arg("description", &selected_kit.description);
        kit_area
            .borrow_mut()
            .state
            .add_text_arg("name", &selected_kit.name);
        children.push(kit_area);

        let starting_abilities = Widget::empty("starting_abilities");
        for ability in &selected_kit.starting_abilities {
            let button =
                Widget::with_defaults(AbilityButton::new(Rc::clone(ability), Rc::clone(class)));
            let icon = Widget::with_theme(Label::empty(), "icon");
            icon.borrow_mut()
                .state
                .add_text_arg("icon", &ability.icon.id());
            Widget::add_child_to(&button, icon);
            Widget::add_child_to(&starting_abilities, button);
        }

        children.push(starting_abilities);

        let mut attr_bonuses: HashMap<Attribute, i32> = HashMap::new();
        for bonus in race.base_stats.iter() {
            if let BonusKind::Attribute { attribute, amount } = bonus.kind {
                attr_bonuses.insert(attribute, amount as i32);
            }
        }

        for attr in Attribute::iter() {
            let value = self.attrs.get(*attr) as i32;
            let max = if self.available > 0 {
                rules.builder_max_attribute
            } else {
                value
            };

            let spinner = Spinner::new(value, rules.builder_min_attribute, max);
            let widget = Widget::with_theme(spinner, &format!("{}_spinner", attr.short_name()));
            widget
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, kind| {
                    let value = Widget::downcast_mut::<Spinner>(kind).value();

                    let (parent, pane) = Widget::parent_mut::<AttributeSelectorPane>(widget);
                    parent.borrow_mut().invalidate_children();
                    pane.attrs.set(*attr, value as u8);
                    pane.set_next_enabled(&parent);
                })));
            children.push(widget);

            let label = Widget::with_theme(Label::empty(), &format!("{}_label", attr.short_name()));
            children.push(label);

            let bonus = Widget::with_theme(Label::empty(), &format!("{}_bonus", attr.short_name()));
            let bonus_value = *attr_bonuses.get(attr).unwrap_or(&0);
            bonus
                .borrow_mut()
                .state
                .add_text_arg("value", &bonus_value.to_string());
            children.push(bonus);

            let total_value = bonus_value + value;
            let total = Widget::with_theme(Label::empty(), &format!("{}_total", attr.short_name()));
            total
                .borrow_mut()
                .state
                .add_text_arg("value", &total_value.to_string());
            children.push(total);
        }

        let points_label = Widget::with_theme(Label::empty(), "points_label");
        points_label
            .borrow_mut()
            .state
            .add_text_arg("points", &self.available.to_string());
        children.push(points_label);

        let amount_label = Widget::with_theme(Label::empty(), "amount_label");
        amount_label
            .borrow_mut()
            .state
            .add_text_arg("points", &self.available.to_string());
        children.push(amount_label);

        children
    }
}

pub struct AbilityButton {
    class: Rc<Class>,
    ability: Rc<Ability>,
}

impl AbilityButton {
    pub fn new(ability: Rc<Ability>, class: Rc<Class>) -> Rc<RefCell<AbilityButton>> {
        Rc::new(RefCell::new(AbilityButton { class, ability }))
    }
}

impl WidgetKind for AbilityButton {
    widget_kind!("ability_button");

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        let hover = Widget::with_theme(TextArea::empty(), "kit_selector_ability_hover");
        add_hover_text_args(
            &mut hover.borrow_mut().state,
            &self.ability,
            &self.class,
            None,
            sulis_state::ability_state::DisabledReason::Enabled,
        );

        Widget::set_mouse_over_widget(
            widget,
            hover,
            widget.borrow().state.inner_right(),
            widget.borrow().state.inner_top(),
        );

        true
    }
}
