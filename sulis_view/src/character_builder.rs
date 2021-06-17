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

mod ability_selector_pane;
use self::ability_selector_pane::AbilitySelectorPane;

mod attribute_selector_pane;
use self::attribute_selector_pane::AttributeSelectorPane;

mod backstory_selector_pane;
use self::backstory_selector_pane::BackstorySelectorPane;

mod class_selector_pane;
use self::class_selector_pane::ClassSelectorPane;

mod color_button;
use self::color_button::ColorButton;

mod cosmetic_selector_pane;
use self::cosmetic_selector_pane::CosmeticSelectorPane;

mod level_up_builder;
use self::level_up_builder::LevelUpBuilder;

mod level_up_finish_pane;
use self::level_up_finish_pane::LevelUpFinishPane;

mod race_selector_pane;
use self::race_selector_pane::RaceSelectorPane;

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::widgets::Button;
use sulis_module::actor::Sex;
use sulis_module::{
    Ability, ActorBuilder, AttributeList, Class, Faction, ImageLayer, InventoryBuilder, Module,
    Race,
};
use sulis_state::EntityState;

use crate::character_window::{get_character_export_filename, write_character_to_file};
use crate::main_menu::CharacterSelector;

pub const NAME: &str = "character_builder";

trait BuilderPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>);

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>);

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>);
}

pub struct CharacterBuilder {
    pub(in crate::character_builder) next: Rc<RefCell<Widget>>,
    pub(in crate::character_builder) prev: Rc<RefCell<Widget>>,
    pub(in crate::character_builder) finish: Rc<RefCell<Widget>>,
    builder_panes: Vec<Rc<RefCell<dyn BuilderPane>>>,
    builder_pane_index: usize,
    // we rely on the builder panes in the above vec having the same
    // index in the children vec of this widget
    builder_set: Rc<dyn BuilderSet>,

    pub race: Option<Rc<Race>>,
    pub class: Option<Rc<Class>>,
    pub kit: Option<usize>,
    pub attributes: Option<AttributeList>,
    pub inventory: Option<InventoryBuilder>,
    pub sex: Option<Sex>,
    pub name: String,
    pub images: HashMap<ImageLayer, String>,
    pub hue: Option<f32>,
    pub skin_color: Option<Color>,
    pub hair_color: Option<Color>,
    pub portrait: Option<String>,

    pub abilities: Vec<Rc<Ability>>,
}

impl CharacterBuilder {
    pub fn new(char_selector_widget: &Rc<RefCell<Widget>>) -> Rc<RefCell<CharacterBuilder>> {
        CharacterBuilder::with(Rc::new(CharacterCreator {
            character_selector_widget: Rc::clone(char_selector_widget),
        }))
    }

    pub fn level_up(pc: Rc<RefCell<EntityState>>) -> Rc<RefCell<CharacterBuilder>> {
        CharacterBuilder::with(Rc::new(LevelUpBuilder { pc }))
    }

    fn with(builder_set: Rc<dyn BuilderSet>) -> Rc<RefCell<CharacterBuilder>> {
        let next = Widget::with_theme(Button::empty(), "next");
        next.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, builder) = Widget::parent_mut::<CharacterBuilder>(widget);
                let cur_pane = Rc::clone(&builder.builder_panes[builder.builder_pane_index]);
                cur_pane.borrow_mut().next(builder, Rc::clone(&parent));
            })));

        let prev = Widget::with_theme(Button::empty(), "previous");
        prev.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, builder) = Widget::parent_mut::<CharacterBuilder>(widget);
                let cur_pane = Rc::clone(&builder.builder_panes[builder.builder_pane_index]);
                cur_pane.borrow_mut().prev(builder, Rc::clone(&parent));
            })));

        let finish = Widget::with_theme(Button::empty(), "finish");
        finish
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, builder) = Widget::parent_mut::<CharacterBuilder>(widget);
                parent.borrow_mut().mark_for_removal();

                let cur_pane = Rc::clone(&builder.builder_panes[builder.builder_pane_index]);
                cur_pane.borrow_mut().next(builder, Rc::clone(&parent));

                let builder_set = Rc::clone(&builder.builder_set);
                builder_set.finish(builder, &parent);
            })));

        Rc::new(RefCell::new(CharacterBuilder {
            builder_set,
            next,
            prev,
            finish,
            builder_panes: Vec::new(),
            builder_pane_index: 0,
            name: String::new(),
            sex: None,
            race: None,
            class: None,
            kit: None,
            hue: None,
            skin_color: None,
            hair_color: None,
            attributes: None,
            inventory: None,
            portrait: None,
            images: HashMap::new(),
            abilities: Vec::new(),
        }))
    }

    pub fn next(&mut self, widget: &Rc<RefCell<Widget>>) {
        self.change_index(widget, 1);
    }

    pub fn prev(&mut self, widget: &Rc<RefCell<Widget>>) {
        self.change_index(widget, -1);
    }

    fn change_index(&mut self, widget: &Rc<RefCell<Widget>>, delta: i32) {
        let cur_child = Rc::clone(&widget.borrow().children[self.builder_pane_index]);
        cur_child.borrow_mut().state.set_visible(false);

        self.builder_pane_index = (self.builder_pane_index as i32 + delta) as usize;
        let cur_pane = Rc::clone(&self.builder_panes[self.builder_pane_index]);
        let cur_child = Rc::clone(&widget.borrow().children[self.builder_pane_index]);
        cur_child.borrow_mut().state.set_visible(true);
        cur_pane.borrow_mut().on_selected(self, cur_child);
    }
}

impl WidgetKind for CharacterBuilder {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<CharacterBuilder>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let builder_set = Rc::clone(&self.builder_set);
        let mut children = builder_set.on_add(self, widget);

        children.append(&mut vec![
            close,
            Rc::clone(&self.next),
            Rc::clone(&self.prev),
            Rc::clone(&self.finish),
        ]);
        children
    }
}

struct CharacterCreator {
    character_selector_widget: Rc<RefCell<Widget>>,
}

impl BuilderSet for CharacterCreator {
    fn on_add(
        &self,
        builder: &mut CharacterBuilder,
        _widget: &Rc<RefCell<Widget>>,
    ) -> Vec<Rc<RefCell<Widget>>> {
        let class_choices = Module::rules().selectable_classes.clone();

        let race_selector_pane = RaceSelectorPane::new();
        let class_selector_pane = ClassSelectorPane::new(class_choices, true, 1);
        let attribute_selector_pane = AttributeSelectorPane::new();
        let backstory_selector_pane = BackstorySelectorPane::new();
        let cosmetic_selector_pane = CosmeticSelectorPane::new();
        let race_sel_widget = Widget::with_defaults(race_selector_pane.clone());
        let class_sel_widget = Widget::with_defaults(class_selector_pane.clone());
        let attr_sel_widget = Widget::with_defaults(attribute_selector_pane.clone());
        let backstory_sel_widget = Widget::with_defaults(backstory_selector_pane.clone());
        let cosmetic_sel_widget = Widget::with_defaults(cosmetic_selector_pane.clone());
        class_sel_widget.borrow_mut().state.set_visible(false);
        attr_sel_widget.borrow_mut().state.set_visible(false);
        backstory_sel_widget.borrow_mut().state.set_visible(false);
        cosmetic_sel_widget.borrow_mut().state.set_visible(false);
        builder.finish.borrow_mut().state.set_visible(false);

        builder.builder_panes.clear();
        builder.builder_pane_index = 0;
        builder.builder_panes.push(race_selector_pane.clone());
        builder.builder_panes.push(class_selector_pane);
        builder.builder_panes.push(attribute_selector_pane);
        builder.builder_panes.push(cosmetic_selector_pane);
        builder.builder_panes.push(backstory_selector_pane);
        race_selector_pane
            .borrow_mut()
            .on_selected(builder, Rc::clone(&race_sel_widget));

        vec![
            race_sel_widget,
            class_sel_widget,
            attr_sel_widget,
            cosmetic_sel_widget,
            backstory_sel_widget,
        ]
    }

    fn finish(&self, builder: &mut CharacterBuilder, _widget: &Rc<RefCell<Widget>>) {
        let (filename, id) = match get_character_export_filename(&builder.name) {
            Err(e) => {
                warn!("{}", e);
                warn!("Unable to save character '{}'", builder.name);
                return;
            }
            Ok(filename) => filename,
        };

        if builder.race.is_none()
            || builder.class.is_none()
            || builder.attributes.is_none()
            || builder.kit.is_none()
        {
            warn!("Unable to save character with undefined stats");
            return;
        }

        let mut levels = HashMap::new();
        levels.insert(builder.class.as_ref().unwrap().id.to_string(), 1);

        let mut abilities = Vec::new();
        for ability in builder.abilities.iter() {
            abilities.push(ability.id.to_string());
        }

        let class = Rc::clone(builder.class.as_ref().unwrap());
        for ability in class.starting_abilities() {
            abilities.push(ability.id.to_string());
        }

        for ability in &class.kits[builder.kit.unwrap()].starting_abilities {
            abilities.push(ability.id.to_string());
        }

        let mut inventory = match builder.inventory {
            None => InventoryBuilder::default(),
            Some(ref inv) => inv.clone(),
        };

        inventory.remove_invalid_items(builder.race.as_ref().unwrap());

        let actor = ActorBuilder {
            id: id.to_string(),
            name: builder.name.to_string(),
            portrait: builder.portrait.clone(),
            race: Some(builder.race.as_ref().unwrap().id.to_string()),
            inline_race: None,
            sex: builder.sex,
            attributes: builder.attributes.unwrap(),
            faction: Some(Faction::Friendly),
            conversation: None,
            images: builder.images.clone(),
            hue: builder.hue,
            hair_color: builder.hair_color,
            skin_color: builder.skin_color,
            inventory,
            levels,
            xp: None,
            reward: None,
            abilities,
            ai: None,
        };

        match write_character_to_file(&filename, &actor) {
            Err(e) => {
                error!("Unable to write actor to file {}", filename);
                error!("{}", e);
            }
            Ok(()) => (),
        }

        self.character_selector_widget
            .borrow_mut()
            .invalidate_children();
        let char_sel = Widget::kind_mut::<CharacterSelector>(&self.character_selector_widget);
        char_sel.set_to_select(id);
    }
}

pub trait BuilderSet {
    fn on_add(
        &self,
        builder: &mut CharacterBuilder,
        widget: &Rc<RefCell<Widget>>,
    ) -> Vec<Rc<RefCell<Widget>>>;

    fn finish(&self, builder: &mut CharacterBuilder, widget: &Rc<RefCell<Widget>>);
}
