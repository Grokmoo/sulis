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
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_core::util::ExtInt;
use sulis_core::widgets::{Button, Label, TextArea};
use sulis_module::{Class, Module};

use crate::bonus_text_arg_handler::{add_bonus_text_args, format_bonus_or_penalty};
use crate::character_builder::{attribute_selector_pane::AbilityButton, BuilderPane};
use crate::{CharacterBuilder, ClassPane};

pub const NAME: &str = "class_selector_pane";

pub struct ClassSelectorPane {
    class_choices: Vec<String>,
    allow_prev: bool,
    level: u32,
    selected_class: Option<Rc<Class>>,
}

impl ClassSelectorPane {
    pub fn new(
        choices: Vec<String>,
        allow_prev: bool,
        level: u32,
    ) -> Rc<RefCell<ClassSelectorPane>> {
        Rc::new(RefCell::new(ClassSelectorPane {
            selected_class: None,
            class_choices: choices,
            allow_prev,
            level,
        }))
    }
}

impl BuilderPane for ClassSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.class = None;
        builder.prev.borrow_mut().state.set_enabled(self.allow_prev);
        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.selected_class.is_some());
        widget.borrow_mut().invalidate_children();
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        let class = match self.selected_class {
            None => return,
            Some(ref class) => class,
        };
        builder.class = Some(Rc::clone(class));
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        if self.allow_prev {
            self.selected_class = None;
            builder.prev(&widget);
        }
    }
}

impl WidgetKind for ClassSelectorPane {
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
        let title = Widget::with_theme(Label::empty(), "title");

        let classes_pane = Widget::empty("classes_pane");
        for class_id in self.class_choices.iter() {
            let class = match Module::class(class_id) {
                None => {
                    warn!("Selectable class '{}' not found", class_id);
                    continue;
                }
                Some(class) => class,
            };

            let class_button = Widget::with_theme(Button::empty(), "class_button");
            {
                let state = &mut class_button.borrow_mut().state;
                state.add_text_arg("name", &class.name);
                state.add_text_arg("level", &format!("{}", self.level));
            }
            class_button
                .borrow_mut()
                .state
                .add_text_arg("name", &class.name);
            if let Some(ref selected_class) = self.selected_class {
                class_button
                    .borrow_mut()
                    .state
                    .set_active(class == *selected_class);
            }

            let class_ref = Rc::clone(&class);
            class_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, pane) = Widget::parent_mut::<ClassSelectorPane>(widget);
                    pane.selected_class = Some(Rc::clone(&class_ref));
                    parent.borrow_mut().invalidate_children();

                    let (_, builder) = Widget::parent_mut::<CharacterBuilder>(&parent);
                    builder.next.borrow_mut().state.set_enabled(true);
                })));

            Widget::add_child_to(&classes_pane, class_button);
        }

        let class = match self.selected_class {
            None => return vec![title, classes_pane],
            Some(ref class) => class,
        };

        let selected = Widget::empty("selected");

        let class_pane = ClassPane::empty();
        class_pane.borrow_mut().set_class(Rc::clone(class));
        Widget::add_child_to(&selected, Widget::with_defaults(class_pane));

        let bonuses = Widget::with_theme(TextArea::empty(), "bonuses");
        add_bonus_text_args(&class.bonuses_per_level, &mut bonuses.borrow_mut().state);
        Widget::add_child_to(&selected, bonuses);

        if self.level == 1 {
            let starting_abilities_title =
                Widget::with_theme(Label::empty(), "starting_abilities_title");
            Widget::add_child_to(&selected, starting_abilities_title);

            let starting_abilities = Widget::empty("starting_abilities");
            for ability in class.starting_abilities() {
                let button =
                    Widget::with_defaults(AbilityButton::new(Rc::clone(ability), Rc::clone(class)));
                let icon = Widget::with_theme(Label::empty(), "icon");
                icon.borrow_mut()
                    .state
                    .add_text_arg("icon", &ability.icon.id());
                Widget::add_child_to(&button, icon);
                Widget::add_child_to(&starting_abilities, button);
            }

            Widget::add_child_to(&selected, starting_abilities);
        } else {
            let upgrades = Widget::with_theme(TextArea::empty(), "upgrades");
            build_upgrades_data(&mut upgrades.borrow_mut().state, class, self.level);

            Widget::add_child_to(&selected, upgrades);
        }

        vec![title, classes_pane, selected]
    }
}

fn build_upgrades_data(state: &mut WidgetState, class: &Class, level: u32) {
    state.add_text_arg("level", &format!("{}", level));

    let mut up_per_day = get_up_per(
        class.group_uses_per_day(level - 1),
        class.group_uses_per_day(level),
    );

    let up_per_enc = get_up_per(
        class.group_uses_per_encounter(level - 1),
        class.group_uses_per_encounter(level),
    );

    let mut uses = HashMap::new();
    for (id, amount_per_enc) in up_per_enc {
        let amount_per_day = up_per_day.remove(&id).unwrap_or(0);
        uses.insert(id, (amount_per_enc, amount_per_day));
    }

    for (id, amount_per_day) in up_per_day {
        uses.insert(id, (0, amount_per_day));
    }

    let mut uses: Vec<_> = uses.into_iter().collect();
    uses.sort_by(|a, b| a.0.cmp(&b.0)); // sort by the ID

    let mut i = 0;
    for (id, (per_enc, per_day)) in uses {
        if per_enc == 0 && per_day == 0 {
            continue;
        }
        state.add_text_arg(&format!("ability_uses_{}_id", i), &id);
        state.add_text_arg(
            &format!("ability_uses_{}_per_enc", i),
            &format_bonus_or_penalty(per_enc),
        );
        state.add_text_arg(
            &format!("ability_uses_{}_per_day", i),
            &format_bonus_or_penalty(per_day),
        );

        i += 1;
    }

    let mut stats: Vec<_> = class.stats_max(level).iter().collect();
    stats.sort_by(|a, b| a.0.cmp(b.0));

    let stats_up = get_up_per_hashmap(
        class.stats_max(level - 1).clone(),
        class.stats_max(level).clone(),
    );

    for (i, (id, amount)) in stats.into_iter().enumerate() {
        let gain = *stats_up.get(id).unwrap_or(&0);
        state.add_text_arg(&format!("stat_{}_id", i), id);
        state.add_text_arg(&format!("stat_{}_total", i), &format!("{}", amount));
        state.add_text_arg(&format!("stat_{}_gain", i), &format!("{}", gain));
    }

    for (i, list) in class.ability_choices(level).iter().enumerate() {
        state.add_text_arg(&format!("choice_{}_name", i), &list.name);
    }
}

fn get_up_per_hashmap(mut prev_per: HashMap<String, ExtInt>, cur_per: HashMap<String, ExtInt>) -> HashMap<String, i32> {
    let mut up_per = HashMap::new();
    for (id, amount) in cur_per {
        let amount = match amount {
            ExtInt::Infinity => continue,
            ExtInt::Int(amount) => amount as i32,
        };

        let old_amount = match prev_per.remove(&id).unwrap_or(ExtInt::Int(0)) {
            ExtInt::Infinity => continue,
            ExtInt::Int(amount) => amount as i32,
        };

        let amount = amount - old_amount;

        up_per.insert(id, amount);
    }

    for (id, amount) in prev_per {
        let amount = match amount {
            ExtInt::Infinity => continue,
            ExtInt::Int(amount) => amount as i32,
        };

        up_per.insert(id, -amount);
    }

    up_per
}

fn get_up_per(prev: &[(String, ExtInt)], cur: &[(String, ExtInt)]) -> HashMap<String, i32> {
    let cur_per: HashMap<_, _> = cur.iter().cloned().collect();
    let prev_per: HashMap<_, _> = prev.iter().cloned().collect();
    get_up_per_hashmap(prev_per, cur_per)
}
