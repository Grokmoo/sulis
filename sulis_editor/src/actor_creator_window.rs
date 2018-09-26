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
use std::collections::HashMap;

use sulis_core::image::{LayeredImage, Image};
use sulis_core::config::Config;
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::resource::write_to_file;
use sulis_core::util::Point;
use sulis_widgets::{Button, Label, list_box, MutuallyExclusiveListBox, InputField};
use sulis_rules::AttributeList;
use sulis_module::{Class, Module, Race, ImageLayer, Faction, Sex, ActorBuilder,
    InventoryBuilder};

pub const NAME: &str = "actor_creator_window";

pub struct ActorCreatorWindow {
    selected_race: Option<Rc<Race>>,
    selected_images: HashMap<ImageLayer, (usize, Rc<Image>)>,
    selected_hue: f32,
    selected_faction: Faction,
    selected_sex: Sex,
    selected_class: Rc<Class>,

    view_pane: Rc<RefCell<Widget>>,
    preview: Option<Rc<LayeredImage>>,

    id_field: Rc<RefCell<InputField>>,
    name_field: Rc<RefCell<InputField>>,
}

impl ActorCreatorWindow {
    pub fn new() -> Rc<RefCell<ActorCreatorWindow>> {
        Rc::new(RefCell::new(ActorCreatorWindow {
            selected_race: None,
            selected_class: Module::all_classes().remove(0),
            selected_images: HashMap::new(),
            view_pane: Widget::empty("view_pane"),
            selected_hue: 0.0,
            selected_faction: Faction::Friendly,
            selected_sex: Sex::Male,
            preview: None,
            id_field: InputField::new("creature01"),
            name_field: InputField::new("Creature"),
        }))
    }

    fn save(&mut self) {
        let id = self.id_field.borrow().text();
        if id.trim().len() == 0 { return; }

        let race = match self.selected_race {
            None => return,
            Some(ref race) => Rc::clone(race),
        };

        let resources_config = Config::resources_config();

        let filename = format!("../{}/{}/actors/{}.yml", resources_config.campaigns_directory,
                               Config::editor_config().module, id);
        info!("Writing created actor to {}", filename);

        let mut images = HashMap::new();
        for (layer, image) in self.selected_images.iter() {
            images.insert(*layer, image.1.id());
        }

        let mut levels = HashMap::new();
        levels.insert(self.selected_class.id.to_string(), 1);

        let actor = ActorBuilder {
            id,
            name: self.name_field.borrow().text(),
            race: race.id.to_string(),
            sex: Some(self.selected_sex),
            portrait: None,
            attributes: AttributeList::new(Module::rules().base_attribute as u8),
            conversation: None,
            faction: Some(self.selected_faction),
            images,
            hue: Some(self.selected_hue),
            hair_color: None,
            skin_color: None,
            inventory: InventoryBuilder::default(),
            levels,
            xp: None,
            reward: None,
            abilities: Vec::new(),
            ai: None,
        };

        match write_to_file(&filename, &actor) {
            Ok(()) => {
                Module::add_actor_to_resources(actor);
            },
            Err(e) => {
                warn!("{}", e);
                warn!("Unable to write created character to file '{}'", filename);
            }
        }
    }

    fn build_preview(&mut self) {
        let mut images = Vec::new();
        for layer in ImageLayer::iter() {
            let image = match self.selected_images.get(&layer) {
                None => continue,
                Some((_, ref image)) => Rc::clone(image),
            };

            images.push((0.0, 0.0, None, image));
        }
        self.preview = Some(Rc::new(LayeredImage::new(images, Some(self.selected_hue))));
    }

    fn populate_hue_pane(&mut self, pane: &Rc<RefCell<Widget>>) {
        let title = Widget::with_theme(Label::empty(), "title");

            let prev = Widget::with_theme(Button::empty(), "prev_button");
            prev.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 2);
                let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);

                let mut hue = window.selected_hue;
                hue -= 0.1;
                if hue < 0.0 { hue = 0.0; }
                window.selected_hue = hue;

                window.build_preview();
            })));

            let next = Widget::with_theme(Button::empty(), "next_button");
            next.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 2);
                let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);

                let mut hue = window.selected_hue;
                hue += 0.1;
                if hue > 1.0 { hue = 1.0; }
                window.selected_hue = hue;

                window.build_preview();
            })));
        Widget::add_children_to(pane, vec![title, prev, next]);
    }

    fn populate_images_pane(&mut self, race: Rc<Race>, pane: &Rc<RefCell<Widget>>) {
        for (layer, images) in race.editor_creator_images() {
            if images.len() == 0 { continue; }

            self.selected_images.entry(layer).or_insert((0, Rc::clone(&images[0])));

            let subpane = Widget::empty("layer_pane");

            let title = Widget::with_theme(Label::new(&format!("{:?}", layer)), "title");
            let prev = Widget::with_theme(Button::empty(), "prev_button");

            let len = images.len();
            let images_ref = images.clone();
            prev.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 3);
                let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);

                let index = window.selected_images.get(&layer).unwrap().0;
                if index > 0 {
                    let index = index - 1;
                    window.selected_images.insert(layer, (index, Rc::clone(&images_ref[index])));
                }
                window.build_preview();
            })));

            let next = Widget::with_theme(Button::empty(), "next_button");

            let images_ref = images.clone();
            next.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::go_up_tree(widget, 3);
                let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);

                let index = window.selected_images.get(&layer).unwrap().0;
                if index < len - 1 {
                    let index = index + 1;
                    window.selected_images.insert(layer, (index, Rc::clone(&images_ref[index])));
                }
                window.build_preview();
            })));

            if images.len() < 2 {
                prev.borrow_mut().state.set_enabled(false);
                next.borrow_mut().state.set_enabled(false);
            }

            Widget::add_children_to(&subpane, vec![title, prev, next]);

            Widget::add_child_to(pane, subpane);
        }
    }
}

impl WidgetKind for ActorCreatorWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
            _widget: &Widget, millis: u32) {
        let preview = match self.preview {
            None => return,
            Some(ref image) => image,
        };

        let child = &self.view_pane.borrow().state;
        let scale_x = 0.8 * child.inner_size.width as f32 / preview.get_width_f32();
        let scale_y = 0.8 * child.inner_size.height as f32 / preview.get_height_f32();
        let x = (child.inner_position.x as f32) / scale_x;
        let y = (child.inner_position.y as f32) / scale_y;

        preview.draw(renderer, scale_x, scale_y, x, y, millis);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let accept = Widget::with_theme(Button::empty(), "accept_button");
        accept.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(widget);
            let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);
            window.save();
        })));
        accept.borrow_mut().state.set_enabled(self.selected_race.is_some());

        let race_pane = Widget::empty("race_pane");
        Widget::add_child_to(&race_pane, Widget::with_theme(Label::empty(), "race_title"));

        let mut entries: Vec<list_box::Entry<Rc<Race>>> = Vec::new();
        for race in Module::all_races() {
            if !race.has_editor_creator_images() { continue; }

            if let Some(ref sel) = self.selected_race {
                if Rc::ptr_eq(sel, &race) {
                    entries.push(list_box::Entry::with_active(race, None));
                    continue;
                }
            }
            entries.push(list_box::Entry::new(race, None));
        }

        let window_ref = Rc::clone(&widget);
        let cb: Rc<Fn(Option<&list_box::Entry<Rc<Race>>>)> = Rc::new(move |active_entry| {
            let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&window_ref);
            match active_entry {
                None => window.selected_race = None,
                Some(ref entry) => window.selected_race = Some(Rc::clone(entry.item())),
            }
            window_ref.borrow_mut().invalidate_children();

        });
        let races_box = MutuallyExclusiveListBox::with_callback(entries, cb);
        Widget::add_child_to(&race_pane, Widget::with_theme(races_box, "races_list"));

        let images_pane = Widget::empty("images_pane");
        if let Some(race) = self.selected_race.clone() {
            self.populate_images_pane(race, &images_pane);
        }

        let hue_pane = Widget::empty("hue_pane");
        if self.selected_race.is_some() {
            self.populate_hue_pane(&hue_pane);
        }

        let id_pane = Widget::empty("id_pane");
        if self.selected_race.is_some() {
            let id_title = Widget::with_theme(Label::empty(), "title");
            let id_widget = Widget::with_theme(self.id_field.clone(), "id_field");
            Widget::add_children_to(&id_pane, vec![id_title, id_widget]);
        }

        let name_pane = Widget::empty("name_pane");
        if self.selected_race.is_some() {
            let name_title = Widget::with_theme(Label::empty(), "title");
            let name_widget = Widget::with_theme(self.name_field.clone(), "name_field");
            Widget::add_children_to(&name_pane, vec![name_title, name_widget]);
        }

        let faction_pane = Widget::empty("faction_pane");
        if self.selected_race.is_some() {
            for faction in Faction::iter() {
                let faction = *faction;
                let widget = Widget::with_theme(Button::empty(), &format!("{:?}", faction));
                if faction == self.selected_faction {
                    widget.borrow_mut().state.set_active(true);
                }
                widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let parent = Widget::go_up_tree(widget, 2);
                    let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);
                    window.selected_faction = faction;
                    parent.borrow_mut().invalidate_children();
                })));
                Widget::add_child_to(&faction_pane, widget);
            }
        }

        let sex_pane = Widget::empty("sex_pane");
        if self.selected_race.is_some() {
            for sex in Sex::iter() {
                let sex = *sex;
                let widget = Widget::with_theme(Button::empty(), &format!("{}", sex));
                if sex == self.selected_sex {
                    widget.borrow_mut().state.set_active(true);
                }
                widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let parent = Widget::go_up_tree(widget, 2);
                    let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);
                    window.selected_sex = sex;
                    parent.borrow_mut().invalidate_children();
                })));
                Widget::add_child_to(&sex_pane, widget);
            }
        }

        let levels_pane = Widget::empty("levels_pane");
        if self.selected_race.is_some() {
            for class in Module::all_classes() {
                let widget = Widget::with_theme(Button::with_text(&class.id), "class_widget");
                if Rc::ptr_eq(&class, &self.selected_class) {
                    widget.borrow_mut().state.set_active(true);
                }
                widget.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let parent = Widget::go_up_tree(widget, 2);
                    let window = Widget::downcast_kind_mut::<ActorCreatorWindow>(&parent);
                    window.selected_class = Rc::clone(&class);
                    parent.borrow_mut().invalidate_children();
                })));
                Widget::add_child_to(&levels_pane, widget);
            }
        }

        self.build_preview();
        vec![title, close, accept, race_pane, images_pane, hue_pane, name_pane,
            id_pane, faction_pane, sex_pane, levels_pane, self.view_pane.clone()]
    }
}
