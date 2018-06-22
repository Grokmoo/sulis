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

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::{animation_state, Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_widgets::{Button, Label};
use sulis_module::{Ability, AbilityList, ability_list::Connect};

use {AbilityPane, CharacterBuilder};
use character_builder::BuilderPane;

pub const NAME: &str = "ability_selector_pane";

pub struct AbilitySelectorPane {
    already_selected: HashSet<Rc<Ability>>,
    prereqs_not_met: HashSet<Rc<Ability>>,
    choices: Rc<AbilityList>,
    selected_ability: Option<Rc<Ability>>,
    index: usize,
}

impl AbilitySelectorPane {
    pub fn new(choices: Rc<AbilityList>, index: usize,
               already_selected: Vec<Rc<Ability>>) -> Rc<RefCell<AbilitySelectorPane>> {
        Rc::new(RefCell::new(AbilitySelectorPane {
            selected_ability: None,
            index,
            choices,
            already_selected: already_selected.into_iter().collect(),
            prereqs_not_met: HashSet::new(),
        }))
    }
}

impl BuilderPane for AbilitySelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        // remove any abilities selected by this pane and subsequent ability panes
        builder.abilities.truncate(self.index);
        builder.prev.borrow_mut().state.set_enabled(true);

        for ability in builder.abilities.iter() {
            self.already_selected.insert(Rc::clone(ability));
        }

        self.prereqs_not_met.clear();
        for entry in self.choices.iter() {
            let ability = &entry.ability;
            if !builder.prereqs_met(ability) {
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
            self.already_selected.remove(ability);
        }
        builder.prev(&widget);
    }
}

struct AbilitiesPane {
    positions: Vec<(f32, f32)>,
    connects: Vec<Vec<Connect>>,
    grid_size: i32,
    grid_border: i32,
    connect_up_image: Rc<Image>,
    connect_down_image: Rc<Image>,
    connect_straight_image: Rc<Image>,
}

impl AbilitiesPane {
    fn new() -> AbilitiesPane {
        AbilitiesPane {
            positions: Vec::new(),
            connects: Vec::new(),
            grid_size: 10,
            grid_border: 1,
            connect_up_image: ResourceSet::get_empty_image(),
            connect_down_image: ResourceSet::get_empty_image(),
            connect_straight_image: ResourceSet::get_empty_image(),
        }
    }
}

impl WidgetKind for AbilitiesPane {
    widget_kind!["abilities_pane"];

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, millis: u32) {
        let grid = self.grid_size as f32;

        let mut draw_list = DrawList::empty_sprite();
        for (index, ref vec) in self.connects.iter().enumerate() {
            for connect in vec.iter() {
                let mut x = widget.state.inner_left() as f32 + self.positions[index].0 * grid;
                let mut y = widget.state.inner_top() as f32 + self.positions[index].1 * grid;

                let mut width = grid;
                x += self.grid_border as f32 * 3.0;
                let image = match connect {
                    &Connect::Up => {
                        &self.connect_up_image
                    }, &Connect::Down => {
                        &self.connect_down_image
                    }, &Connect::Straight => {
                        &self.connect_straight_image
                    }, &Connect::LongUp => {
                        y -= grid / 2.0;
                        x += grid / 2.0 - self.grid_border as f32 * 3.0;
                        &self.connect_up_image
                    }, &Connect::LongDown => {
                        y += grid / 2.0;
                        x += grid / 2.0 - self.grid_border as f32 * 3.0;
                        &self.connect_down_image
                    }, &Connect::Straight2x => {
                        width += grid;
                        &self.connect_straight_image
                    }, &Connect::Straight3x => {
                        width += 2.0 * grid;
                        &self.connect_straight_image
                    }
                };

                image.append_to_draw_list(&mut draw_list, &widget.state.animation_state, x, y,
                                          width, grid, millis);
            }
        }

        renderer.draw(draw_list);
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref theme) = widget.theme {
            self.grid_size = theme.get_custom_or_default("grid_size", 10);
            self.grid_border = theme.get_custom_or_default("grid_border", 1);
            if let Some(ref image_id) = theme.custom.get("connect_up") {
                self.connect_up_image = ResourceSet::get_image_else_empty(image_id);
            }
            if let Some(ref image_id) = theme.custom.get("connect_down") {
                self.connect_down_image = ResourceSet::get_image_else_empty(image_id);
            }
            if let Some(ref image_id) = theme.custom.get("connect_straight") {
                self.connect_straight_image = ResourceSet::get_image_else_empty(image_id);
            }
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

        let pane = Rc::new(RefCell::new(AbilitiesPane::new()));
        let abilities_pane = Widget::with_defaults(pane.clone());
        abilities_pane.borrow_mut().state.background = Some(Rc::clone(&self.choices.background));
        for entry in self.choices.iter() {
            let ability = &entry.ability;
            let position = entry.position;
            pane.borrow_mut().positions.push(position);
            pane.borrow_mut().connects.push(entry.connect.clone());

            let ability_button = Widget::with_theme(Button::empty(), "ability_button");

            if self.already_selected.contains(ability) {
                ability_button.borrow_mut().state.animation_state.add(animation_state::Kind::Custom1);
            }

            if self.prereqs_not_met.contains(ability) {
                ability_button.borrow_mut().state.animation_state.add(animation_state::Kind::Custom2);
            }

            let icon = Widget::with_theme(Label::empty(), "icon");
            icon.borrow_mut().state.add_text_arg("icon", &ability.icon.id());
            Widget::add_child_to(&ability_button, icon);

            if let Some(ref selected_ability) = self.selected_ability {
                ability_button.borrow_mut().state.set_active(*ability == *selected_ability);
            }

            let enable_next = self.already_selected.contains(ability) || self.prereqs_not_met.contains(ability);
            let ability_ref = Rc::clone(&ability);
            ability_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(&widget, 2);
                let pane = Widget::downcast_kind_mut::<AbilitySelectorPane>(&parent);
                pane.selected_ability = Some(Rc::clone(&ability_ref));
                parent.borrow_mut().invalidate_children();

                let builder_widget = Widget::get_parent(&parent);
                let builder = Widget::downcast_kind_mut::<CharacterBuilder>(&builder_widget);
                builder.next.borrow_mut().state.set_enabled(!enable_next);
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

        if self.already_selected.contains(ability) {
            details.borrow_mut().state.add_text_arg("already_owned", "true");
        }

        vec![title, ability_pane_widget, abilities_pane]
    }
}
