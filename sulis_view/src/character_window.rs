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
use std::fs;
use std::io::Error;
use std::rc::Rc;

use chrono::prelude::*;

use sulis_core::config;
use sulis_core::resource::write_to_file;
use sulis_core::ui::{Callback, Widget, WidgetKind, WidgetState};
use sulis_core::util::ExtInt;
use sulis_core::widgets::{Button, ScrollPane, TextArea};
use sulis_module::{
    ActorBuilder, Attribute, DamageKind, InventoryBuilder, ItemListEntrySaveState, ItemSaveState,
    Module, QuickSlot, Slot,
};
use sulis_state::{ActorState, ChangeListener, Effect, EntityState, GameState};

use crate::ability_pane::add_ability_text_args;
use crate::item_button::add_bonus_text_args;
use crate::CharacterBuilder;

pub const NAME: &str = "character_window";

enum ActivePane {
    Character,
    Ability {
        show_passives: bool,
    },
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
        self.character
            .borrow_mut()
            .actor
            .listeners
            .add(ChangeListener::invalidate(NAME, widget));

        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(
            NAME,
            Box::new(move |entity| {
                let entity = match entity {
                    Some(entity) => entity,
                    None => return,
                };
                let window = Widget::kind_mut::<CharacterWindow>(&widget_ref);
                window.character = Rc::clone(entity);
                widget_ref.borrow_mut().invalidate_children();
            }),
        ));

        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<CharacterWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let char_ref = Rc::clone(&self.character);
        let level_up = Widget::with_theme(Button::empty(), "level_up");
        level_up.borrow_mut().state.set_visible(false);
        level_up
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let root = Widget::get_root(&widget);
                let window =
                    Widget::with_defaults(CharacterBuilder::level_up(Rc::clone(&char_ref)));
                window.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, window);
            })));
        level_up
            .borrow_mut()
            .state
            .set_enabled(!GameState::is_combat_active());

        let char_pane = Widget::with_theme(Button::empty(), "char_pane_button");
        char_pane
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<CharacterWindow>(widget);
                window.active_pane = ActivePane::Character;
                parent.borrow_mut().invalidate_children();
            })));

        let abilities_pane = Widget::with_theme(Button::empty(), "abilities_pane_button");
        abilities_pane
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<CharacterWindow>(widget);
                window.active_pane = ActivePane::Ability { show_passives: true };
                parent.borrow_mut().invalidate_children();
            })));

        let effects_pane = Widget::with_theme(Button::empty(), "effects_pane_button");
        effects_pane
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<CharacterWindow>(widget);
                window.active_pane = ActivePane::Effect;
                parent.borrow_mut().invalidate_children();
            })));

        let cur_pane = match self.active_pane {
            ActivePane::Character => {
                char_pane.borrow_mut().state.set_active(true);
                level_up
                    .borrow_mut()
                    .state
                    .set_visible(self.character.borrow_mut().actor.has_level_up());
                let is_pc = Rc::ptr_eq(&self.character, &GameState::player());
                create_details_text_box(&self.character.borrow().actor, is_pc)
            }
            ActivePane::Ability { show_passives } => {
                abilities_pane.borrow_mut().state.set_active(true);
                create_abilities_pane(&self.character.borrow().actor, show_passives)
            }
            ActivePane::Effect => {
                effects_pane.borrow_mut().state.set_active(true);
                create_effects_pane(&mut self.character.borrow_mut().actor)
            }
        };

        vec![
            close,
            cur_pane,
            level_up,
            char_pane,
            abilities_pane,
            effects_pane,
        ]
    }
}

pub fn get_inventory(pc: &ActorState) -> InventoryBuilder {
    let coins = GameState::party_coins();

    let stash = GameState::party_stash();
    let items = stash
        .borrow()
        .items()
        .iter()
        .map(|(qty, item)| ItemListEntrySaveState::new(*qty, &item.item))
        .collect();

    let equipped = Slot::iter()
        .map(|slot| (*slot, pc.inventory().equipped(*slot)))
        .filter(|(_, item)| item.is_some())
        .map(|(slot, item)| (slot, ItemSaveState::new(&item.unwrap().item)))
        .collect();

    let quick = QuickSlot::iter()
        .map(|slot| (*slot, pc.inventory().quick(*slot)))
        .filter(|(_, item)| item.is_some())
        .map(|(slot, item)| (slot, ItemSaveState::new(&item.unwrap().item)))
        .collect();

    InventoryBuilder::new(equipped, quick, coins, items)
}

fn export_character(pc: &ActorState) {
    let (filename, id) = match get_character_export_filename(&pc.actor.name) {
        Err(e) => {
            warn!("{}", e);
            warn!("Unable to export character '{}'", pc.actor.name);
            return;
        }
        Ok(filename) => filename,
    };

    let portrait = match pc.actor.portrait {
        None => None,
        Some(ref img) => Some(img.id().to_string()),
    };

    let abilities = pc
        .actor
        .abilities
        .iter()
        .map(|a| a.ability.id.to_string())
        .collect();
    let levels = pc
        .actor
        .levels
        .iter()
        .map(|(class, level)| (class.id.to_string(), *level))
        .collect();

    let inventory = get_inventory(&pc);

    let actor = ActorBuilder {
        id,
        name: pc.actor.name.to_string(),
        portrait,
        race: pc.actor.race.id.to_string(),
        sex: Some(pc.actor.sex),
        attributes: pc.actor.attributes,
        faction: Some(pc.actor.faction()),
        conversation: None,
        images: pc.actor.builder_images.clone(),
        hue: pc.actor.hue.clone(),
        hair_color: pc.actor.hair_color.clone(),
        skin_color: pc.actor.skin_color.clone(),
        abilities,
        levels,
        inventory,
        xp: Some(pc.xp()),
        reward: None,
        ai: None,
    };

    match write_character_to_file(&filename, &actor) {
        Err(e) => {
            warn!("{}", e);
            warn!("Unable to write character '{}' to file", pc.actor.name);
        }
        Ok(()) => (),
    }
}

pub fn write_character_to_file(filename: &str, actor: &ActorBuilder) -> Result<(), Error> {
    info!("Writing character '{}' to '{}'", actor.id, filename);
    write_to_file(filename, actor)
}

pub fn get_character_export_filename(name: &str) -> Result<(String, String), Error> {
    let utc = Utc::now();

    let mut path = config::USER_DIR.clone();
    path.push("characters");
    let dir = path.as_path();
    let id = format!("pc_{}_{}", name, utc.format("%Y%m%d-%H%M%S"));
    let filename = format!("{}/{}.yml", dir.to_string_lossy(), id);

    if let Err(e) = fs::create_dir_all(dir) {
        error!(
            "Unable to create characters directory '{}'",
            dir.to_string_lossy()
        );
        return Err(e);
    }

    Ok((filename, id))
}

pub fn create_abilities_pane(pc: &ActorState, show_passive: bool) -> Rc<RefCell<Widget>> {
    let abilities = Widget::empty("abilities");

    let passive = Widget::with_theme(Button::empty(), "show_passives");
    passive.borrow_mut().state.set_active(show_passive);
    passive.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
        let result = !show_passive;
        let (parent, window) = Widget::parent_mut::<CharacterWindow>(&widget);
        window.active_pane = ActivePane::Ability {
            show_passives: result,
        };
        parent.borrow_mut().invalidate_children();
    })));
    Widget::add_child_to(&abilities, passive);

    let details_scroll = ScrollPane::new();
    let details_widget = Widget::with_theme(details_scroll.clone(), "details_pane");
    let details = Widget::with_theme(TextArea::empty(), "details");
    details_scroll.borrow().add_to_content(Rc::clone(&details));

    let scrollpane = ScrollPane::new();
    let list = Widget::with_theme(scrollpane.clone(), "list");
    for ability in pc.actor.abilities.iter() {
        if !show_passive && ability.ability.active.is_none() { continue; }

        let level = ability.level;
        let ability = &ability.ability;
        let button = Widget::with_theme(Button::empty(), "ability_button");
        {
            let state = &mut button.borrow_mut().state;
            state.add_text_arg("icon", &ability.icon.id());
            state.add_text_arg("name", &ability.name);
        }

        let ability_ref = Rc::clone(&ability);
        let details_widget_ref = Rc::clone(&details_widget);
        let details_ref = Rc::clone(&details);
        button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |_, _| {
                add_ability_text_args(&mut details_ref.borrow_mut().state, &ability_ref);
                details_ref
                    .borrow_mut()
                    .state
                    .add_text_arg("owned_level", &(level + 1).to_string());
                details_widget_ref.borrow_mut().invalidate_layout();
            })));

        scrollpane.borrow().add_to_content(button);
    }

    Widget::add_child_to(&abilities, list);
    Widget::add_child_to(&abilities, details_widget);
    abilities
}

pub fn create_effects_pane(pc: &mut ActorState) -> Rc<RefCell<Widget>> {
    let scrollpane = ScrollPane::new();
    let effects = Widget::with_theme(scrollpane.clone(), "effects");

    let mgr = GameState::turn_manager();
    let mut mgr = mgr.borrow_mut();
    for index in pc.effects_iter() {
        let effect = mgr.effect_mut(*index);
        let widget = Widget::with_theme(TextArea::empty(), "effect");

        add_effect_text_args(effect, &mut widget.borrow_mut().state);

        let widget_ref = Rc::clone(&widget);
        effect.listeners.add(ChangeListener::new(
            "effects",
            Box::new(move |effect| {
                add_effect_text_args(effect, &mut widget_ref.borrow_mut().state);
                widget_ref.borrow_mut().invalidate_layout();
            }),
        ));

        scrollpane.borrow().add_to_content(widget);
    }

    effects
}

fn add_effect_text_args(effect: &Effect, widget_state: &mut WidgetState) {
    widget_state.add_text_arg("name", effect.name());
    match effect.remaining_duration_rounds() {
        ExtInt::Infinity => widget_state.add_text_arg("active_mode", "true"),
        ExtInt::Int(_) => {
            widget_state.add_text_arg(
                "total_duration",
                &effect.total_duration_rounds().to_string(),
            );
            widget_state.add_text_arg(
                "remaining_duration",
                &effect.remaining_duration_rounds().to_string(),
            );
        }
    }
    add_bonus_text_args(&effect.bonuses(), widget_state);
}

fn add_if_nonzero(state: &mut WidgetState, index: usize, name: &str, value: f32) {
    if value == 0.0 {
        return;
    }

    state.add_text_arg(&format!("{}_{}", index, name), &value.to_string());
}

pub fn create_details_text_box(pc: &ActorState, is_pc: bool) -> Rc<RefCell<Widget>> {
    let details = Widget::with_theme(TextArea::empty(), "details");
    {
        let export = Widget::with_theme(Button::empty(), "export");
        export.borrow_mut().state.set_visible(is_pc);
        export.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let player = GameState::player();
            widget.borrow_mut().state.set_enabled(false);
            export_character(&player.borrow().actor);
        })));
        Widget::add_child_to(&details, export);

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
            state.add_text_arg(
                attribute.short_name(),
                &stats.attributes.get(*attribute).to_string(),
            )
        }

        let mut index = 0;
        for &(ref class, level) in pc.actor.levels.iter() {
            state.add_text_arg(&format!("class_{}", index), &class.name);
            state.add_text_arg(&format!("level_{}", index), &level.to_string());

            index += 1;
        }

        for (index, attack) in stats.attacks.iter().enumerate() {
            state.add_text_arg(
                &format!("{}_damage_min", index),
                &attack.damage.min().to_string(),
            );
            state.add_text_arg(
                &format!("{}_damage_max", index),
                &attack.damage.max().to_string(),
            );
            if attack.damage.ap() > 0 {
                state.add_text_arg(
                    &format!("{}_armor_penetration", index),
                    &attack.damage.ap().to_string(),
                );
            }

            add_if_nonzero(
                state,
                index,
                "spell_accuracy",
                attack.bonuses.spell_accuracy as f32,
            );
            add_if_nonzero(
                state,
                index,
                "ranged_accuracy",
                attack.bonuses.ranged_accuracy as f32,
            );
            add_if_nonzero(
                state,
                index,
                "melee_accuracy",
                attack.bonuses.melee_accuracy as f32,
            );
            add_if_nonzero(
                state,
                index,
                "crit_chance",
                attack.bonuses.crit_chance as f32,
            );
            add_if_nonzero(
                state,
                index,
                "hit_threshold",
                attack.bonuses.hit_threshold as f32,
            );
            add_if_nonzero(
                state,
                index,
                "graze_threshold",
                attack.bonuses.graze_threshold as f32,
            );
            add_if_nonzero(
                state,
                index,
                "crit_multiplier",
                attack.bonuses.crit_multiplier,
            );
            add_if_nonzero(
                state,
                index,
                "hit_multiplier",
                attack.bonuses.hit_multiplier,
            );
            add_if_nonzero(
                state,
                index,
                "graze_multiplier",
                attack.bonuses.graze_multiplier,
            );
        }

        state.add_text_arg("reach", &stats.attack_distance().to_string());
        state.add_text_arg("cur_hp", &pc.hp().to_string());
        state.add_text_arg("max_hp", &stats.max_hp.to_string());
        state.add_text_arg("cur_ap", &pc.ap().to_string());

        state.add_text_arg("cur_xp", &pc.xp().to_string());
        state.add_text_arg(
            "next_xp",
            &rules
                .get_xp_for_next_level(pc.actor.total_level)
                .to_string(),
        );

        state.add_text_arg("initiative", &stats.initiative.to_string());
        state.add_text_arg("flanking_angle", &stats.flanking_angle.to_string());
        state.add_text_arg("caster_level", &stats.caster_level.to_string());

        state.add_text_arg("armor", &stats.armor.base().to_string());
        for kind in DamageKind::iter() {
            if !stats.armor.differs_from_base(*kind) {
                continue;
            }

            state.add_text_arg(
                &format!("armor_{}", kind).to_lowercase(),
                &stats.armor.amount(*kind).to_string(),
            );
        }

        for kind in DamageKind::iter() {
            let amount = stats.resistance.amount(*kind);
            if amount == 0 {
                continue;
            }

            state.add_text_arg(
                &format!("resistance_{}", kind).to_lowercase(),
                &amount.to_string(),
            );
        }

        state.add_text_arg("melee_accuracy", &stats.melee_accuracy.to_string());
        state.add_text_arg("ranged_accuracy", &stats.ranged_accuracy.to_string());
        state.add_text_arg("spell_accuracy", &stats.spell_accuracy.to_string());
        state.add_text_arg("defense", &stats.defense.to_string());
        state.add_text_arg("fortitude", &stats.fortitude.to_string());
        state.add_text_arg("reflex", &stats.reflex.to_string());
        state.add_text_arg("will", &stats.will.to_string());
        state.add_text_arg("concealment", &stats.concealment.to_string());
        state.add_text_arg("concealment_ignore", &stats.concealment_ignore.to_string());
        state.add_text_arg("crit_chance", &stats.crit_chance.to_string());
        state.add_text_arg("hit_threshold", &stats.hit_threshold.to_string());
        state.add_text_arg("graze_threshold", &stats.graze_threshold.to_string());
        state.add_text_arg("crit_multiplier", &format!("{:.2}", stats.crit_multiplier));
        state.add_text_arg("hit_multiplier", &format!("{:.2}", stats.hit_multiplier));
        state.add_text_arg(
            "graze_multiplier",
            &format!("{:.2}", stats.graze_multiplier),
        );
        state.add_text_arg("movement_rate", &format!("{:.2}", stats.movement_rate));
    }
    details
}
