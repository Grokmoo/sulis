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

use sulis_core::image::{Image, LayeredImage};
use sulis_core::io::GraphicsRenderer;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::{Offset, Point, Scale};
use sulis_core::widgets::{Button, InputField, Label, ScrollDirection, ScrollPane};
use sulis_module::actor::Sex;
use sulis_module::{ImageLayer, ImageLayerSet, ItemState, Race, Slot};

use crate::character_builder::{BuilderPane, ColorButton};
use crate::CharacterBuilder;

pub const NAME: &str = "cosmetic_selector_pane";

pub struct CosmeticSelectorPane {
    preview: Rc<RefCell<Widget>>,

    race: Option<Rc<Race>>,
    items: Vec<(Slot, ItemState)>,
    sex: Sex,
    name: String,
    hair_index: Option<usize>,
    beard_index: Option<usize>,
    hair_color: Option<Color>,
    skin_color: Option<Color>,
    hue: Option<f32>,
    portrait: Option<Rc<dyn Image>>,

    preview_image: Option<Rc<LayeredImage>>,
}

impl CosmeticSelectorPane {
    pub fn new() -> Rc<RefCell<CosmeticSelectorPane>> {
        let preview = Widget::with_theme(Label::empty(), "preview");
        Rc::new(RefCell::new(CosmeticSelectorPane {
            sex: Sex::Male,
            race: None,
            items: Vec::new(),
            name: String::new(),
            hair_index: None,
            beard_index: None,
            hue: Some(0.0),
            hair_color: None,
            skin_color: None,
            portrait: None,
            preview,
            preview_image: None,
        }))
    }

    fn build_preview(&mut self) {
        let race = match self.race {
            None => return,
            Some(ref race) => race,
        };

        let images = self.build_images();
        let image_layers = match ImageLayerSet::merge(race.default_images(), self.sex, images) {
            Err(_) => return,
            Ok(image) => image,
        };

        let mut insert: HashMap<ImageLayer, Rc<dyn Image>> = HashMap::new();
        for (slot, item) in self.items.iter() {
            if *slot == Slot::Head {
                continue;
            } // don't show helmet so we can see hair and ears
            if race.disabled_slots.contains(slot) {
                continue;
            }

            let iter = match &item.item.equippable {
                Some(equip) => {
                    if equip.slot == *slot {
                        item.image_iter()
                    } else if equip.alternate_slot.unwrap() == *slot {
                        item.alt_image_iter()
                    } else {
                        unreachable!()
                    }
                }
                None => unreachable!(),
            };

            iter.for_each(|(layer, image)| {
                insert.insert(*layer, Rc::clone(image));
            });
        }

        let images_list =
            image_layers.get_list_with(self.sex, race, self.hair_color, self.skin_color, insert);
        self.preview_image = Some(Rc::new(LayeredImage::new(images_list, self.hue)));
    }

    fn build_images(&self) -> HashMap<ImageLayer, String> {
        let mut images = HashMap::new();

        let race = match self.race {
            None => return images,
            Some(ref race) => race,
        };

        if let Some(index) = self.hair_index {
            let hair_string = &race.hair_selections[index];
            images.insert(ImageLayer::Hair, hair_string.to_string());
        }

        if let Some(index) = self.beard_index {
            let beard_string = &race.beard_selections[index];
            images.insert(ImageLayer::Beard, beard_string.to_string());
        }

        images
    }

    fn set_finish_enabled(&self, widget: &Rc<RefCell<Widget>>) {
        let (_, builder) = Widget::parent_mut::<CharacterBuilder>(widget);
        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.name.len() > 1 && self.portrait.is_some());
    }
}

impl BuilderPane for CosmeticSelectorPane {
    fn on_selected(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        self.race = builder.race.clone();
        if let Some(ref race) = self.race {
            if !race.hair_selections.is_empty() {
                self.hair_index = Some(0);
            } else {
                self.hair_index = None;
            }
            self.beard_index = None;
            if !race.hair_colors.is_empty() {
                self.hair_color = Some(race.hair_colors[0]);
            } else {
                self.hair_color = None;
            }
            if !race.skin_colors.is_empty() {
                self.skin_color = Some(race.skin_colors[0]);
            } else {
                self.skin_color = None;
            }
        }

        self.items.clear();
        if let Some(ref inventory) = builder.inventory {
            for (slot, item) in inventory.equipped_iter() {
                self.items.push((slot, item));
            }
        }

        builder
            .next
            .borrow_mut()
            .state
            .set_enabled(self.name.len() > 1 && self.portrait.is_some());
        self.build_preview();
        widget.borrow_mut().invalidate_children();
    }

    fn next(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        builder.sex = Some(self.sex);
        builder.name = self.name.to_string();
        builder.images = self.build_images();
        builder.hue = self.hue;
        builder.skin_color = self.skin_color;
        builder.hair_color = self.hair_color;
        builder.portrait = self.portrait.as_ref().map(|i| i.id());
        builder.next(&widget);
    }

    fn prev(&mut self, builder: &mut CharacterBuilder, widget: Rc<RefCell<Widget>>) {
        self.portrait = None;
        self.hue = Some(0.0);
        builder.prev(&widget);
    }
}

impl WidgetKind for CosmeticSelectorPane {
    fn get_name(&self) -> &str {
        NAME
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        _pixel_size: Point,
        _widget: &Widget,
        millis: u32,
    ) {
        let race = match self.race {
            None => return,
            Some(ref race) => race,
        };

        let preview = match self.preview_image {
            None => return,
            Some(ref image) => image,
        };

        let child = self.preview.borrow();
        let scale = Scale {
            x: 0.8 * child.state.inner_width() as f32 / preview.get_width_f32(),
            y: 0.8 * child.state.inner_height() as f32 / preview.get_height_f32(),
        };
        let offset = Offset {
            x: (child.state.inner_left() as f32) / scale.x + race.ticker_offset.0,
            y: (child.state.inner_top() as f32) / scale.y + race.ticker_offset.1 - 0.25,
        };

        preview.draw(renderer, offset, scale, millis);
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let race = match &self.race {
            None => return vec![],
            Some(race) => Rc::clone(race),
        };

        self.build_preview();
        let title = Widget::with_theme(Label::empty(), "title");

        let name_label = Widget::with_theme(Label::empty(), "name_label");
        let name_field = Widget::with_theme(InputField::new(&self.name), "name_field");
        name_field
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);
                let field = Widget::downcast::<InputField>(kind);

                cosmetic_pane.name = field.text.to_string();
                cosmetic_pane.set_finish_enabled(&parent);
            })));

        let random_name_button = Widget::with_theme(Button::empty(), "random_name_button");
        match self.sex {
            Sex::Male => {
                if race.male_random_names.is_empty() {
                    random_name_button.borrow_mut().state.set_visible(false);
                }
            }
            Sex::Female => {
                if race.female_random_names.is_empty() {
                    random_name_button.borrow_mut().state.set_visible(false);
                }
            }
        }

        random_name_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);

                let race = match pane.race {
                    None => return,
                    Some(ref race) => race,
                };

                let name = race.random_name(&pane.sex);
                pane.name = name.to_string();
                pane.set_finish_enabled(&parent);

                parent.borrow_mut().invalidate_children();
            })));

        let male_button = Widget::with_theme(Button::empty(), "male_button");
        male_button
            .borrow_mut()
            .state
            .set_active(self.sex == Sex::Male);
        male_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);
                cosmetic_pane.sex = Sex::Male;
                cosmetic_pane.beard_index = None;
                parent.borrow_mut().invalidate_children();
            })));

        let female_button = Widget::with_theme(Button::empty(), "female_button");
        female_button
            .borrow_mut()
            .state
            .set_active(self.sex == Sex::Female);
        female_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);
                cosmetic_pane.sex = Sex::Female;
                cosmetic_pane.beard_index = None;
                parent.borrow_mut().invalidate_children();
            })));

        let hair_label = Widget::with_theme(Label::empty(), "hair_label");
        let next_hair = Widget::with_theme(Button::empty(), "next_hair");
        next_hair
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);

                let race = match cosmetic_pane.race {
                    None => return,
                    Some(ref race) => race,
                };
                match cosmetic_pane.hair_index {
                    None => cosmetic_pane.hair_index = Some(0),
                    Some(index) => {
                        if index == race.hair_selections.len() - 1 {
                            cosmetic_pane.hair_index = None;
                        } else {
                            cosmetic_pane.hair_index = Some(index + 1);
                        }
                    }
                }
                parent.borrow_mut().invalidate_children();
            })));
        let prev_hair = Widget::with_theme(Button::empty(), "prev_hair");
        prev_hair
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);

                let race = match cosmetic_pane.race {
                    None => return,
                    Some(ref race) => race,
                };
                match cosmetic_pane.hair_index {
                    None => cosmetic_pane.hair_index = Some(race.hair_selections.len() - 1),
                    Some(index) => {
                        if index == 0 {
                            cosmetic_pane.hair_index = None;
                        } else {
                            cosmetic_pane.hair_index = Some(index - 1);
                        }
                    }
                }
                parent.borrow_mut().invalidate_children();
            })));

        let beard_label = Widget::with_theme(Label::empty(), "beard_label");
        let next_beard = Widget::with_theme(Button::empty(), "next_beard");
        next_beard
            .borrow_mut()
            .state
            .set_enabled(self.sex == Sex::Male);
        next_beard
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);

                let race = match cosmetic_pane.race {
                    None => return,
                    Some(ref race) => race,
                };
                match cosmetic_pane.beard_index {
                    None => cosmetic_pane.beard_index = Some(0),
                    Some(index) => {
                        if index == race.beard_selections.len() - 1 {
                            cosmetic_pane.beard_index = None;
                        } else {
                            cosmetic_pane.beard_index = Some(index + 1);
                        }
                    }
                }
                parent.borrow_mut().invalidate_children();
            })));
        let prev_beard = Widget::with_theme(Button::empty(), "prev_beard");
        prev_beard
            .borrow_mut()
            .state
            .set_enabled(self.sex == Sex::Male);
        prev_beard
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, cosmetic_pane) = Widget::parent_mut::<CosmeticSelectorPane>(widget);

                let race = match cosmetic_pane.race {
                    None => return,
                    Some(ref race) => race,
                };
                match cosmetic_pane.beard_index {
                    None => cosmetic_pane.beard_index = Some(race.beard_selections.len() - 1),
                    Some(index) => {
                        if index == 0 {
                            cosmetic_pane.beard_index = None;
                        } else {
                            cosmetic_pane.beard_index = Some(index - 1);
                        }
                    }
                }
                parent.borrow_mut().invalidate_children();
            })));

        let color_label = Widget::with_theme(Label::empty(), "color_label");
        let color_panel = Widget::empty("color_panel");
        let mut hue = 0.0;
        while hue < 1.0 {
            let color = hue_to_color(hue);
            let color_button = Widget::with_defaults(ColorButton::new(color));

            color_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, cosmetic_pane) =
                        Widget::parent_mut::<CosmeticSelectorPane>(widget);

                    cosmetic_pane.hue = Some(hue);
                    parent.borrow_mut().invalidate_children();
                })));
            Widget::add_child_to(&color_panel, color_button);

            hue += 0.05;
        }

        let skin_color_label = Widget::with_theme(Label::empty(), "skin_color_label");
        let skin_color_panel = Widget::empty("skin_color_panel");
        for color in race.skin_colors.iter() {
            let color = *color;
            let color_button = Widget::with_defaults(ColorButton::new(color));
            color_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, cosmetic_pane) =
                        Widget::parent_mut::<CosmeticSelectorPane>(widget);
                    cosmetic_pane.skin_color = Some(color);
                    parent.borrow_mut().invalidate_children();
                })));
            Widget::add_child_to(&skin_color_panel, color_button);
        }

        let hair_color_label = Widget::with_theme(Label::empty(), "hair_color_label");
        let hair_color_panel = Widget::empty("hair_color_panel");
        for color in race.hair_colors.iter() {
            let color = *color;
            let color_button = Widget::with_defaults(ColorButton::new(color));
            color_button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, cosmetic_pane) =
                        Widget::parent_mut::<CosmeticSelectorPane>(widget);
                    cosmetic_pane.hair_color = Some(color);
                    parent.borrow_mut().invalidate_children();
                })));
            Widget::add_child_to(&hair_color_panel, color_button);
        }

        let portrait_label = Widget::with_theme(Label::empty(), "portrait_label");

        let portrait_button = Widget::with_theme(Button::empty(), "portrait_button");
        if let Some(ref image) = self.portrait {
            portrait_button.borrow_mut().state.foreground = Some(Rc::clone(image));
        }

        let race_ref = Rc::clone(&race);
        portrait_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let (parent, _) = Widget::parent::<CosmeticSelectorPane>(widget);

                let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
                let pop_up = Widget::with_theme(scrollpane.clone(), "portrait_selector");
                pop_up.borrow_mut().state.set_modal(true);
                pop_up.borrow_mut().state.modal_remove_on_click_outside = true;

                for portrait_id in race_ref.portrait_selections.iter() {
                    let portrait = match ResourceSet::image(portrait_id) {
                        None => {
                            warn!("Invalid race portrait selection '{}'", portrait_id);
                            continue;
                        }
                        Some(portrait) => portrait,
                    };

                    let button = Widget::with_theme(Button::empty(), "portrait_button");
                    button
                        .borrow_mut()
                        .state
                        .add_callback(portrait_selector_button_callback(&portrait, &parent));
                    button.borrow_mut().state.foreground = Some(portrait);
                    scrollpane.borrow().add_to_content(button);
                }

                let root = Widget::get_root(widget);
                Widget::add_child_to(&root, pop_up);
            })));

        if race.hair_selections.is_empty() {
            next_hair.borrow_mut().state.set_enabled(false);
            prev_hair.borrow_mut().state.set_enabled(false);
        }

        if race.beard_selections.is_empty() {
            next_beard.borrow_mut().state.set_enabled(false);
            prev_beard.borrow_mut().state.set_enabled(false);
        }

        if race.hair_selections.is_empty() && race.beard_selections.is_empty() {
            hair_color_label.borrow_mut().state.set_visible(false);
            hair_color_panel.borrow_mut().state.set_visible(false);
        }

        vec![
            title,
            name_field,
            name_label,
            random_name_button,
            Rc::clone(&self.preview),
            male_button,
            female_button,
            hair_label,
            next_hair,
            prev_hair,
            beard_label,
            next_beard,
            prev_beard,
            color_label,
            color_panel,
            skin_color_panel,
            skin_color_label,
            hair_color_label,
            hair_color_panel,
            portrait_button,
            portrait_label,
        ]
    }
}

fn hue_to_color(hue: f32) -> Color {
    let k = [1.0, 2.0 / 3.0, 1.0 / 3.0];
    let mut frac = [hue + k[0], hue + k[1], hue + k[2]];
    frac.iter_mut().for_each(|e| {
        if *e > 1.0 {
            *e -= 1.0;
        }
    });

    let p = [
        (frac[0] * 6.0 - 3.0).abs(),
        (frac[1] * 6.0 - 3.0).abs(),
        (frac[2] * 6.0 - 3.0).abs(),
    ];

    let mut res = [p[0] - 1.0, p[1] - 1.0, p[2] - 1.0];
    res.iter_mut().for_each(|e| {
        *e = e.clamp(0.0, 1.0);
    });

    Color::new(res[0], res[1], res[2], 1.0)
}

fn portrait_selector_button_callback(
    portrait: &Rc<dyn Image>,
    pane_widget: &Rc<RefCell<Widget>>,
) -> Callback {
    let pane_widget_ref = Rc::clone(pane_widget);
    let image = Rc::clone(portrait);
    Callback::new(Rc::new(move |widget, _| {
        let cosmetic_pane = Widget::kind_mut::<CosmeticSelectorPane>(&pane_widget_ref);
        cosmetic_pane.portrait = Some(Rc::clone(&image));
        pane_widget_ref.borrow_mut().invalidate_children();
        cosmetic_pane.set_finish_enabled(&pane_widget_ref);

        let (parent, _) = Widget::parent::<ScrollPane>(widget);
        parent.borrow_mut().mark_for_removal();
    }))
}
