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
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, TextArea};
use sulis_state::ActorState;

pub const NAME: &str = "character_window";

pub struct CharacterWindow {
    character: Rc<RefCell<EntityState>>,
}

impl CharacterWindow {
    pub fn new(character: &Rc<RefCell<EntityState>>) -> Rc<RefCell<CharacterWindow>> {
        Rc::new(RefCell::new(CharacterWindow {
            character: Rc::clone(character)
        }))
    }
}

impl WidgetKind for CharacterWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

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
        title.borrow_mut().state.add_text_arg("name", &self.character.borrow().actor.actor.id);

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let details = create_details_text_box(&self.character.borrow().actor);
        vec![title, close, details]
    }
}

pub fn create_details_text_box(pc: &ActorState) -> Rc<RefCell<Widget>> {
    let details = Widget::with_theme(TextArea::empty(), "details");
    {
        let state = &mut details.borrow_mut().state;
        let stats = &pc.stats;

        state.add_text_arg("name", &pc.actor.name);
        state.add_text_arg("race", &pc.actor.race.name);
        state.add_text_arg("sex", &pc.actor.sex.to_string());

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
