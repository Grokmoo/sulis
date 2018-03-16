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

use sulis_rules::{Attribute, DamageKind};
use sulis_state::{ChangeListener, EntityState};
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_widgets::{Button, Label, TextArea};
use sulis_module::Module;
use sulis_state::{ActorState, Effect};

use CharacterBuilder;
use inventory_window::add_bonus_text_args;

pub const NAME: &str = "character_window";

pub struct CharacterWindow {
    character: Rc<RefCell<EntityState>>,

    char_pane_active: bool,
}

impl CharacterWindow {
    pub fn new(character: &Rc<RefCell<EntityState>>) -> Rc<RefCell<CharacterWindow>> {
        Rc::new(RefCell::new(CharacterWindow {
            character: Rc::clone(character),
            char_pane_active: true,
        }))
    }
}

impl WidgetKind for CharacterWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

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

        let title = Widget::with_theme(Label::empty(), "title");
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.name);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let level_up = Widget::with_theme(Button::empty(), "level_up");
        level_up.borrow_mut().state.set_visible(self.character.borrow_mut().actor.has_level_up());
        level_up.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let root = Widget::get_root(&widget);
            let window = Widget::with_defaults(CharacterBuilder::level_up());
            window.borrow_mut().state.set_modal(true);
            Widget::add_child_to(&root, window);
        })));

        let char_pane = Widget::with_theme(Button::empty(), "char_pane_button");
        char_pane.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&parent);
            window.char_pane_active = true;
            parent.borrow_mut().invalidate_children();
        })));

        let effects_pane = Widget::with_theme(Button::empty(), "effects_pane_button");
        effects_pane.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CharacterWindow>(&parent);
            window.char_pane_active = false;
            parent.borrow_mut().invalidate_children();
        })));

        let cur_pane = if self.char_pane_active {
            char_pane.borrow_mut().state.set_active(true);
            create_details_text_box(&self.character.borrow().actor)
        } else {
            effects_pane.borrow_mut().state.set_active(true);
            create_effects_pane(&mut self.character.borrow_mut().actor)
        };

        vec![title, close, cur_pane, level_up, char_pane, effects_pane]
    }
}

pub fn create_effects_pane(pc: &mut ActorState) -> Rc<RefCell<Widget>> {
    let effects = Widget::empty("effects");

    for effect in pc.effects_iter_mut() {
        let widget = Widget::with_theme(TextArea::empty(), "effect");

        add_effect_text_args(&effect, &mut widget.borrow_mut().state);

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
    widget_state.add_text_arg("total_duration",
                              &effect.total_duration_rounds().to_string());
    widget_state.add_text_arg("remaining_duration",
                              &effect.remaining_duration_rounds().to_string());
    add_bonus_text_args(&effect.bonuses(), widget_state);
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
        }

        state.add_text_arg("reach", &stats.attack_distance().to_string());
        state.add_text_arg("cur_hp", &pc.hp().to_string());
        state.add_text_arg("max_hp", &stats.max_hp.to_string());
        state.add_text_arg("cur_ap", &pc.ap().to_string());

        state.add_text_arg("cur_xp", &pc.xp().to_string());
        state.add_text_arg("next_xp", &rules.get_xp_for_next_level(pc.actor.total_level).to_string());

        state.add_text_arg("initiative", &stats.initiative.to_string());

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
    }
    details
}
