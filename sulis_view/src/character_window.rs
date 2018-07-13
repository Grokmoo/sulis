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

use sulis_core::util::ExtInt;
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_rules::{Attribute, DamageKind};
use sulis_widgets::{Button, Label, TextArea};
use sulis_module::Module;
use sulis_state::{ActorState, Effect, GameState, ChangeListener, EntityState};

use CharacterBuilder;
use ability_pane::add_ability_text_args;
use item_button::{add_bonus_text_args};

pub const NAME: &str = "character_window";

enum ActivePane {
    Character,
    Ability,
    Effect,
}

pub struct CharacterWindow {
    character: Rc<RefCell<EntityState>>,

    active_pane: ActivePane,
}

impl CharacterWindow {
    pub fn new(character: &Rc<RefCell<EntityState>>) -> Rc<RefCell<CharacterWindow>> {
        Rc::new(RefCell::new(CharacterWindow {
            character: Rc::clone(character),
            active_pane: ActivePane::Character,
        }))
    }
}

impl WidgetKind for CharacterWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();
    }

    fn on_remove(&mut self) {
        self.character.borrow_mut().actor.listeners.remove(NAME);
        debug!("Removed character window.");
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.character.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate(NAME, widget));

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |entity| {
            let entity = match entity {
                Some(entity) => entity,
                None => return,
            };
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&widget_ref);
            window.character = Rc::clone(entity);
            widget_ref.borrow_mut().invalidate_children();
        })));

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.name);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let char_ref = Rc::clone(&self.character);
        let level_up = Widget::with_theme(Button::empty(), "level_up");
        level_up.borrow_mut().state.set_visible(self.character.borrow_mut().actor.has_level_up());
        level_up.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let root = Widget::get_root(&widget);
            let window = Widget::with_defaults(CharacterBuilder::level_up(Rc::clone(&char_ref)));
            window.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, window);
        })));

        let char_pane = Widget::with_theme(Button::empty(), "char_pane_button");
        char_pane.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&parent);
            window.active_pane = ActivePane::Character;
            parent.borrow_mut().invalidate_children();
        })));

        let abilities_pane = Widget::with_theme(Button::empty(), "abilities_pane_button");
        abilities_pane.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&parent);
            window.active_pane = ActivePane::Ability;
            parent.borrow_mut().invalidate_children();
        })));

        let effects_pane = Widget::with_theme(Button::empty(), "effects_pane_button");
        effects_pane.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&parent);
            window.active_pane = ActivePane::Effect;
            parent.borrow_mut().invalidate_children();
        })));

        let cur_pane = match self.active_pane {
            ActivePane::Character => {
                char_pane.borrow_mut().state.set_active(true);
                create_details_text_box(&self.character.borrow().actor)
            }, ActivePane::Ability => {
                abilities_pane.borrow_mut().state.set_active(true);
                create_abilities_pane(&self.character.borrow().actor)
            }, ActivePane::Effect => {
                effects_pane.borrow_mut().state.set_active(true);
                create_effects_pane(&mut self.character.borrow_mut().actor)
            }
        };

        vec![title, close, cur_pane, level_up, char_pane, abilities_pane, effects_pane]
    }
}

pub fn create_abilities_pane(pc: &ActorState) -> Rc<RefCell<Widget>> {
    let abilities = Widget::empty("abilities");

    let details = Widget::with_theme(TextArea::empty(), "details");

    let list = Widget::empty("list");
    for ability in pc.actor.abilities.iter() {
        let level = ability.level;
        let ability = &ability.ability;
        let button = Widget::with_theme(Button::empty(), "ability_button");
        button.borrow_mut().state.add_text_arg("icon", &ability.icon.id());

        let ability_ref = Rc::clone(&ability);
        let details_ref = Rc::clone(&details);
        button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_, _| {
            add_ability_text_args(&mut details_ref.borrow_mut().state, &ability_ref);
            details_ref.borrow_mut().state.add_text_arg("owned_level", &(level + 1).to_string());
            details_ref.borrow_mut().invalidate_layout();
        })));

        Widget::add_child_to(&list, button);
    }

    Widget::add_child_to(&abilities, list);
    Widget::add_child_to(&abilities, details);
    abilities
}

pub fn create_effects_pane(pc: &mut ActorState) -> Rc<RefCell<Widget>> {
    let effects = Widget::empty("effects");

    let area_state = GameState::area_state();
    let mut area_state = area_state.borrow_mut();
    for index in pc.effects_iter() {
        let effect = area_state.effect_mut(*index);
        let widget = Widget::with_theme(TextArea::empty(), "effect");

        add_effect_text_args(effect, &mut widget.borrow_mut().state);

        let widget_ref = Rc::clone(&widget);
        effect.listeners.add(ChangeListener::new("effects", Box::new(move |effect| {
            add_effect_text_args(effect, &mut widget_ref.borrow_mut().state);
            widget_ref.borrow_mut().invalidate_layout();
        })));

        Widget::add_child_to(&effects, widget);
    }

    effects
}

fn add_effect_text_args(effect: &Effect, widget_state: &mut WidgetState) {
    widget_state.add_text_arg("name", effect.name());
    match effect.remaining_duration_rounds() {
        ExtInt::Infinity => widget_state.add_text_arg("active_mode", "true"),
        ExtInt::Int(_) => {
            widget_state.add_text_arg("total_duration",
                                      &effect.total_duration_rounds().to_string());
            widget_state.add_text_arg("remaining_duration",
                                      &effect.remaining_duration_rounds().to_string());
        }
    }
    add_bonus_text_args(&effect.bonuses(), widget_state);
}

fn add_if_nonzero(state: &mut WidgetState, index: usize, name: &str, value: f32) {
    if value == 0.0 { return; }

    state.add_text_arg(&format!("{}_{}", index, name), &value.to_string());
}

pub fn create_details_text_box(pc: &ActorState) -> Rc<RefCell<Widget>> {
    let details = Widget::with_theme(TextArea::empty(), "details");
    {
        let rules = Module::rules();
        let state = &mut details.borrow_mut().state;
        let stats = &pc.stats;

        state.add_text_arg("name", &pc.actor.name);
        state.add_text_arg("race", &pc.actor.race.name);
        state.add_text_arg("sex", &pc.actor.sex.to_string());

        if let Some(ref portrait) = pc.actor.portrait {
            state.add_text_arg("portrait", &portrait.id());
        }

        for attribute in Attribute::iter() {
            state.add_text_arg(attribute.short_name(), &stats.attributes.get(*attribute).to_string())
        }

        let mut index = 0;
        for &(ref class, level) in pc.actor.levels.iter() {
            state.add_text_arg(&format!("class_{}", index), &class.name);
            state.add_text_arg(&format!("level_{}", index), &level.to_string());

            index += 1;
        }

        for (index, attack) in stats.attacks.iter().enumerate() {
            state.add_text_arg(&format!("{}_damage_min", index)
                               , &attack.damage.min().to_string());
            state.add_text_arg(&format!("{}_damage_max", index)
                               , &attack.damage.max().to_string());
            if attack.damage.ap() > 0 {
                state.add_text_arg(&format!("{}_armor_penetration", index)
                                   , &attack.damage.ap().to_string());
            }
            add_if_nonzero(state, index, "accuracy", attack.bonuses.accuracy as f32);
            add_if_nonzero(state, index, "crit_threshold", attack.bonuses.crit_threshold as f32);
            add_if_nonzero(state, index, "hit_threshold", attack.bonuses.hit_threshold as f32);
            add_if_nonzero(state, index, "graze_threshold", attack.bonuses.graze_threshold as f32);
            add_if_nonzero(state, index, "crit_multiplier", attack.bonuses.crit_multiplier);
            add_if_nonzero(state, index, "hit_multiplier", attack.bonuses.hit_multiplier);
            add_if_nonzero(state, index, "graze_multiplier", attack.bonuses.graze_multiplier);
        }

        state.add_text_arg("reach", &stats.attack_distance().to_string());
        state.add_text_arg("cur_hp", &pc.hp().to_string());
        state.add_text_arg("max_hp", &stats.max_hp.to_string());
        state.add_text_arg("cur_ap", &pc.ap().to_string());

        state.add_text_arg("cur_xp", &pc.xp().to_string());
        state.add_text_arg("next_xp", &rules.get_xp_for_next_level(pc.actor.total_level).to_string());

        state.add_text_arg("initiative", &stats.initiative.to_string());
        state.add_text_arg("flanking_angle", &stats.flanking_angle.to_string());

        state.add_text_arg("armor", &stats.armor.base().to_string());
        for kind in DamageKind::iter() {
            if !stats.armor.differs_from_base(*kind) { continue; }

            state.add_text_arg(&format!("armor_{}", kind).to_lowercase(),
            &stats.armor.amount(*kind).to_string());
        }

        state.add_text_arg("accuracy", &stats.accuracy.to_string());
        state.add_text_arg("defense", &stats.defense.to_string());
        state.add_text_arg("fortitude", &stats.fortitude.to_string());
        state.add_text_arg("reflex", &stats.reflex.to_string());
        state.add_text_arg("will", &stats.will.to_string());
        state.add_text_arg("concealment", &stats.concealment.to_string());
        state.add_text_arg("concealment_ignore", &stats.concealment_ignore.to_string());
        state.add_text_arg("crit_threshold", &stats.crit_threshold.to_string());
        state.add_text_arg("hit_threshold", &stats.hit_threshold.to_string());
        state.add_text_arg("graze_threshold", &stats.graze_threshold.to_string());
        state.add_text_arg("crit_multiplier", &format!("{:.2}", stats.crit_multiplier));
        state.add_text_arg("hit_multiplier", &format!("{:.2}", stats.hit_multiplier));
        state.add_text_arg("graze_multiplier", &format!("{:.2}", stats.graze_multiplier));
        state.add_text_arg("movement_rate", &format!("{:.2}", stats.movement_rate));
    }
    details
}
